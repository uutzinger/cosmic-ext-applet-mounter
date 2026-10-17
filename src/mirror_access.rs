// SPDX-License-Identifier: MIT
//! Keep host engine paths separate from portal-authorized local I/O paths.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::model::{Connection, ConnectionMode};
use crate::process::{
    CommandExecutionMode, CommandRequest, CommandRunner, Executable, RuntimeCommandRunner,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalMirrorPaths {
    pub mirror: PathBuf,
    pub recovery: PathBuf,
}

impl LocalMirrorPaths {
    pub fn native(connection: &Connection) -> Result<Self, String> {
        let ConnectionMode::OfflineMirror(options) = &connection.mode else {
            return Err(crate::fl!("mirror-access-unavailable"));
        };
        Ok(Self {
            mirror: connection.local_path.clone(),
            recovery: options.recovery_directory.clone(),
        })
    }

    pub async fn resolve(
        connection: &Connection,
        cancel: CancellationToken,
    ) -> Result<Self, String> {
        if cancel.is_cancelled() {
            return Err(crate::fl!("mirror-access-unavailable"));
        }
        let paths = Self::native(connection)?;
        if CommandExecutionMode::detect_current() == CommandExecutionMode::Native {
            return Ok(paths);
        }
        let resolver = DocumentsOrigin;
        Ok(Self {
            mirror: resolve_directory(&paths.mirror, &resolver, cancel.child_token()).await?,
            recovery: resolve_directory(&paths.recovery, &resolver, cancel).await?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryGrant {
    host_path: PathBuf,
    portal_path: PathBuf,
}

fn absolute_clean(path: &Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|p| matches!(p, Component::RootDir | Component::Normal(_)))
}

fn document_id(path: &Path) -> Option<String> {
    if !absolute_clean(path) {
        return None;
    }
    let parts = path
        .iter()
        .map(|p| p.to_str())
        .collect::<Option<Vec<_>>>()?;
    let rest = match parts.as_slice() {
        ["/", "run", "user", uid, "doc", rest @ ..] if uid.bytes().all(|b| b.is_ascii_digit()) => {
            rest
        }
        ["/", "run", "flatpak", "doc", rest @ ..] => rest,
        _ => return None,
    };
    let id = match rest {
        ["by-app", app, id, _] if *app == crate::config::APP_ID => *id,
        [id, _] => *id,
        _ => return None,
    };
    (!id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
    .then(|| id.to_owned())
}

fn grant_file(host: &Path) -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or_else(|| crate::fl!("mirror-access-unavailable"))?;
    let hash = Sha256::digest(host.as_os_str().as_encoded_bytes());
    Ok(PathBuf::from(home)
        .join(".config/cosmic-ext-applet-mounter/portal-directory-grants")
        .join(format!("{hash:x}.json")))
}

/// Called only after the folder picker resolved the selected document's origin.
/// This records an existing grant; it neither creates nor broadens permissions.
pub fn remember_directory(host: &Path, portal: &Path) -> Result<(), String> {
    if CommandExecutionMode::detect_current() == CommandExecutionMode::Native
        || document_id(portal).is_none()
    {
        return Ok(());
    }
    if !absolute_clean(host) || portal.file_name() != host.file_name() {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    let file = grant_file(host)?;
    let parent = file.parent().expect("grant directory");
    if file.ancestors().any(|p| p.is_symlink()) {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    std::fs::create_dir_all(parent).map_err(|_| crate::fl!("mirror-access-unavailable"))?;
    let temporary = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec(&DirectoryGrant {
        host_path: host.to_path_buf(),
        portal_path: portal.to_path_buf(),
    })
    .map_err(|_| crate::fl!("mirror-access-unavailable"))?;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let result = (|| {
        let mut out = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        out.write_all(&bytes)?;
        out.sync_all()?;
        std::fs::rename(&temporary, &file)
    })();
    let _ = std::fs::remove_file(temporary);
    result.map_err(|_| crate::fl!("mirror-access-unavailable"))
}

type OriginFuture<'a> = Pin<Box<dyn Future<Output = Result<PathBuf, String>> + Send + 'a>>;
trait DirectoryOrigin: Send + Sync {
    fn origin<'a>(&'a self, portal: &'a Path, cancel: CancellationToken) -> OriginFuture<'a>;
}

async fn resolve_directory(
    host: &Path,
    resolver: &dyn DirectoryOrigin,
    cancel: CancellationToken,
) -> Result<PathBuf, String> {
    if !absolute_clean(host) {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| crate::fl!("mirror-access-unavailable"))?;
    // These exact app-owned namespaces are already shared by the manifest.
    if [
        ".config/cosmic-ext-applet-mounter",
        ".cache/cosmic-ext-applet-mounter",
        ".local/state/cosmic-ext-applet-mounter",
    ]
    .iter()
    .any(|root| host.starts_with(home.join(root)))
    {
        return Ok(host.to_path_buf());
    }
    let file = grant_file(host)?;
    let grant = read_grant(&file).await?;
    let path = validate_grant(host, &grant, resolver, cancel.child_token()).await?;
    require_writable_directory(&path).await?;
    if cancel.is_cancelled() {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    Ok(path)
}

async fn require_writable_directory(path: &Path) -> Result<(), String> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|_| crate::fl!("mirror-access-unavailable"))?;
    if !metadata.is_dir() || metadata.permissions().readonly() {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    Ok(())
}

async fn read_grant(file: &Path) -> Result<DirectoryGrant, String> {
    if file.ancestors().any(|p| p.is_symlink()) {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    let metadata = tokio::fs::metadata(file)
        .await
        .map_err(|_| crate::fl!("mirror-access-select-directories"))?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    use tokio::io::AsyncReadExt;
    let input = tokio::fs::File::open(file)
        .await
        .map_err(|_| crate::fl!("mirror-access-unavailable"))?;
    let mut data = Vec::new();
    input
        .take(16 * 1024 + 1)
        .read_to_end(&mut data)
        .await
        .map_err(|_| crate::fl!("mirror-access-unavailable"))?;
    if data.len() > 16 * 1024 {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    serde_json::from_slice(&data).map_err(|_| crate::fl!("mirror-access-unavailable"))
}

async fn validate_grant(
    host: &Path,
    grant: &DirectoryGrant,
    resolver: &dyn DirectoryOrigin,
    cancel: CancellationToken,
) -> Result<PathBuf, String> {
    if cancel.is_cancelled() {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    if grant.host_path != host
        || !absolute_clean(host)
        || document_id(&grant.portal_path).is_none()
        || grant.portal_path.file_name() != host.file_name()
    {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    tokio::select! {
        () = cancel.cancelled() => Err(crate::fl!("mirror-access-unavailable")),
        result = tokio::time::timeout(Duration::from_secs(8), resolver.origin(&grant.portal_path, cancel.child_token())) => {
            let origin = result.map_err(|_| crate::fl!("mirror-access-unavailable"))??;
            if origin != host || cancel.is_cancelled() { return Err(crate::fl!("mirror-access-unavailable")); }
            Ok(grant.portal_path.clone())
        }
    }
}

struct DocumentsOrigin;
impl DirectoryOrigin for DocumentsOrigin {
    fn origin<'a>(&'a self, portal: &'a Path, cancel: CancellationToken) -> OriginFuture<'a> {
        Box::pin(async move {
            let error = || crate::fl!("mirror-access-unavailable");
            let id = document_id(portal).ok_or_else(error)?;
            let connection = zbus::Connection::session().await.map_err(|_| error())?;
            let proxy = zbus::Proxy::new(
                &connection,
                "org.freedesktop.portal.Documents",
                "/org/freedesktop/portal/documents",
                "org.freedesktop.portal.Documents",
            )
            .await
            .map_err(|_| error())?;
            let version: u32 = proxy.get_property("version").await.map_err(|_| error())?;
            if version < 4 {
                return Err(error());
            }
            if version >= 5 {
                let paths: HashMap<String, Vec<u8>> = proxy
                    .call("GetHostPaths", &(vec![id],))
                    .await
                    .map_err(|_| error())?;
                let mut bytes = paths.into_values().next().ok_or_else(error)?;
                if bytes.last() == Some(&0) {
                    bytes.pop();
                }
                if bytes.contains(&0) {
                    return Err(error());
                }
                use std::os::unix::ffi::OsStringExt;
                return Ok(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
            }
            // Preserve the existing reviewed portal-v4 exception, only after
            // the typed property confirms GetHostPaths is unavailable.
            let request = CommandRequest::new(Executable::Flatpak)
                .watch_host_bus()
                .arg("document-info")
                .and_then(|r| r.arg(portal.as_os_str()))
                .map_err(|_| error())?
                .with_timeout(Duration::from_secs(5))
                .with_output_limit(64 * 1024);
            let out = RuntimeCommandRunner::detect_current()
                .run(request, cancel)
                .await
                .map_err(|_| error())?;
            if out.stdout.truncated || out.stdout.invalid_utf8 {
                return Err(error());
            }
            out.stdout
                .text
                .lines()
                .find_map(|line| {
                    line.trim()
                        .strip_prefix("origin:")
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(PathBuf::from)
                })
                .ok_or_else(error)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_only_missing_or_linked_directories_are_not_write_authority() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("directory");
        assert!(require_writable_directory(&folder).await.is_err());
        std::fs::create_dir(&folder).unwrap();
        assert!(require_writable_directory(&folder).await.is_ok());
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o500)).unwrap();
        assert!(require_writable_directory(&folder).await.is_err());
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700)).unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&folder, &link).unwrap();
        assert!(require_writable_directory(&link).await.is_err());
    }

    #[tokio::test]
    async fn malformed_oversized_or_linked_grants_never_supply_paths() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("grant.json");
        assert!(read_grant(&file).await.is_err());
        for bytes in [b"incomplete {".to_vec(), vec![b'x'; 16 * 1024 + 1]] {
            std::fs::write(&file, bytes).unwrap();
            assert!(read_grant(&file).await.is_err());
        }
        let grant = DirectoryGrant {
            host_path: "/home/example/Test".into(),
            portal_path: "/run/user/1000/doc/abc/Test".into(),
        };
        std::fs::write(&file, serde_json::to_vec(&grant).unwrap()).unwrap();
        assert_eq!(read_grant(&file).await.unwrap().host_path, grant.host_path);
        let linked = root.path().join("link.json");
        std::os::unix::fs::symlink(&file, &linked).unwrap();
        assert!(read_grant(&linked).await.is_err());
    }
    struct Origin(Option<PathBuf>);
    impl DirectoryOrigin for Origin {
        fn origin<'a>(&'a self, _: &'a Path, _: CancellationToken) -> OriginFuture<'a> {
            Box::pin(async move { self.0.clone().ok_or_else(|| "revoked".into()) })
        }
    }
    #[tokio::test]
    async fn grants_revalidate_origin_and_never_fall_back_to_shadow_paths() {
        let host = Path::new("/home/example/Cloud/Test");
        let grant = DirectoryGrant {
            host_path: host.into(),
            portal_path: "/run/user/1000/doc/abc123/Test".into(),
        };
        assert_eq!(
            validate_grant(
                host,
                &grant,
                &Origin(Some(host.into())),
                CancellationToken::new()
            )
            .await
            .unwrap(),
            grant.portal_path
        );
        assert!(
            validate_grant(
                host,
                &grant,
                &Origin(Some("/home/example/Other".into())),
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert!(
            validate_grant(host, &grant, &Origin(None), CancellationToken::new())
                .await
                .is_err()
        );
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(
            validate_grant(host, &grant, &Origin(Some(host.into())), cancel)
                .await
                .is_err()
        );
        let invalid = DirectoryGrant {
            host_path: host.into(),
            portal_path: "/home/example/Cloud/Test".into(),
        };
        assert!(
            validate_grant(
                host,
                &invalid,
                &Origin(Some(host.into())),
                CancellationToken::new()
            )
            .await
            .is_err()
        );
    }
    #[test]
    fn portal_paths_reject_traversal_and_accept_app_scoped_documents() {
        assert!(document_id(Path::new("/run/user/1000/doc/abc/../foreign")).is_none());
        assert!(document_id(Path::new("relative/doc/abc/Test")).is_none());
        assert_eq!(
            document_id(&PathBuf::from(format!(
                "/run/user/1000/doc/by-app/{}/abc/Test",
                crate::config::APP_ID
            ))),
            Some("abc".into())
        );
    }
}
