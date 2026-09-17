// SPDX-License-Identifier: MIT

//! The panel owns runtime state. Standalone windows only send requests.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cosmic::iced::{Subscription, futures::SinkExt, stream};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use cosmic_ext_applet_mounter::model::PreloadSettings;

pub const RUNTIME_NAME: &str = "io.github.uutzinger.cosmic-ext-applet-mounter.Runtime";
const SETTINGS_NAME: &str = "io.github.uutzinger.cosmic-ext-applet-mounter.Settings";
const OBJECT_PATH: &str = "/io/github/uutzinger/CloudMounter";
const RUNTIME_INTERFACE: &str = "io.github.uutzinger.CloudMounter.Runtime";
const SETTINGS_INTERFACE: &str = "io.github.uutzinger.CloudMounter.Settings";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
    Status,
    Refresh,
    SetUnmount(bool),
    SetRestore(bool),
    SetPreload(PreloadSettings),
    ConfigurationChanged,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Status {
    pub unmount_before_sleep: bool,
    pub restore_after_wake: bool,
    pub preload: PreloadSettings,
    pub sleep_status: Option<String>,
}

type Response = Result<Status, String>;

/// Iced messages are cloneable; only the first completion consumes the reply.
#[derive(Debug, Clone)]
pub struct Reply(Arc<Mutex<Option<oneshot::Sender<Response>>>>);
impl Reply {
    pub fn channel() -> (Self, oneshot::Receiver<Response>) {
        let (sender, receiver) = oneshot::channel();
        (Self(Arc::new(Mutex::new(Some(sender)))), receiver)
    }

    pub fn complete(&self, result: Response) {
        if let Some(sender) = self.0.lock().expect("IPC reply").take() {
            let _ = sender.send(result);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready,
    ClientReady,
    Request(Command, Reply),
    Activate,
    DuplicateWindow,
    Error(String),
}

struct RuntimeInterface(mpsc::Sender<Event>);
#[zbus::interface(name = "io.github.uutzinger.CloudMounter.Runtime")]
impl RuntimeInterface {
    async fn request(&self, command: &str) -> zbus::fdo::Result<String> {
        let command = serde_json::from_str(command)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(format!("{e}")))?;
        let (reply, receiver) = Reply::channel();
        self.0
            .send(Event::Request(command, reply))
            .await
            .map_err(|_| zbus::fdo::Error::Failed("Applet closed".into()))?;
        let status = tokio::time::timeout(Duration::from_secs(120), receiver)
            .await
            .map_err(|_| {
                zbus::fdo::Error::Failed(
                    "Applet request timed out; check current status before retrying".into(),
                )
            })?
            .map_err(|_| zbus::fdo::Error::Failed("Applet closed before replying".into()))?
            .map_err(zbus::fdo::Error::Failed)?;
        serde_json::to_string(&status).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }
}

struct SettingsInterface(mpsc::Sender<Event>);
#[zbus::interface(name = "io.github.uutzinger.CloudMounter.Settings")]
impl SettingsInterface {
    async fn activate(&self) -> zbus::fdo::Result<()> {
        self.0
            .send(Event::Activate)
            .await
            .map_err(|_| zbus::fdo::Error::Failed("Settings window closed".into()))
    }
}

pub async fn request(command: Command) -> Response {
    let timeout = if command == Command::Status { 5 } else { 125 };
    tokio::time::timeout(Duration::from_secs(timeout), async {
        let connection = zbus::connection::Builder::session()
            .map_err(|e| e.to_string())?
            .method_timeout(Duration::from_secs(timeout))
            .build()
            .await
            .map_err(|e| e.to_string())?;
        let proxy = zbus::Proxy::new(&connection, RUNTIME_NAME, OBJECT_PATH, RUNTIME_INTERFACE)
            .await
            .map_err(|e| e.to_string())?;
        let json = serde_json::to_string(&command).map_err(|e| e.to_string())?;
        let response: String = proxy
            .call("Request", &(json,))
            .await
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&response).map_err(|e| e.to_string())
    })
    .await
    .map_err(|_| "Applet request timed out; check current status before retrying".to_string())?
    .map_err(|e| format!("Could not contact or update the running applet: {e}"))
}

pub async fn activate_settings() -> bool {
    tokio::time::timeout(Duration::from_secs(2), async {
        let connection = zbus::Connection::session().await?;
        let proxy =
            zbus::Proxy::new(&connection, SETTINGS_NAME, OBJECT_PATH, SETTINGS_INTERFACE).await?;
        proxy.call::<_, _, ()>("Activate", &()).await
    })
    .await
    .is_ok_and(|result: zbus::Result<()>| result.is_ok())
}

pub fn runtime_subscription() -> Subscription<Event> {
    Subscription::run(|| stream::channel(16, |output| serve(false, output)))
}
pub fn settings_subscription() -> Subscription<Event> {
    Subscription::run(|| stream::channel(16, |output| serve(true, output)))
}

async fn serve(settings: bool, mut output: cosmic::iced::futures::channel::mpsc::Sender<Event>) {
    let (sender, mut receiver) = mpsc::channel(16);
    let result = async {
        let builder = zbus::connection::Builder::session()?;
        if settings {
            builder
                .allow_name_replacements(false)
                .replace_existing_names(false)
                .name(SETTINGS_NAME)?
                .serve_at(OBJECT_PATH, SettingsInterface(sender))?
                .build()
                .await
        } else {
            builder
                .allow_name_replacements(false)
                .replace_existing_names(false)
                .name(RUNTIME_NAME)?
                .serve_at(OBJECT_PATH, RuntimeInterface(sender))?
                .build()
                .await
        }
    }
    .await;
    match result {
        Ok(connection) => {
            // Keep the bus name and interfaces alive until this window closes.
            let _connection = connection;
            let _ = output.send(Event::Ready).await;
            while let Some(event) = receiver.recv().await {
                if output.send(event).await.is_err() {
                    break;
                }
            }
        }
        Err(error) => {
            if settings && activate_settings().await {
                let _ = output.send(Event::DuplicateWindow).await;
            } else if !settings && request(Command::Status).await.is_ok() {
                // More than one panel may contain the applet. Only one instance
                // owns the runtime bus name; the others remain usable clients.
                let _ = output.send(Event::ClientReady).await;
            } else {
                let _ = output
                    .send(Event::Error(format!(
                        "Window communication unavailable: {error}"
                    )))
                    .await;
            }
            std::future::pending::<()>().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Child, Stdio};

    struct TestBus(Child);
    impl Drop for TestBus {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    // Never uses the desktop's bus, configuration, mounts, or sleep service.
    #[tokio::test]
    async fn private_bus_routes_requests_errors_and_singleton_activation() {
        let runtime_dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime_dir.path(), std::fs::Permissions::from_mode(0o700))
            .unwrap();
        let child = std::process::Command::new("dbus-daemon")
            .env("XDG_RUNTIME_DIR", runtime_dir.path())
            .args([
                "--session",
                "--nofork",
                "--print-address=1",
                "--address=unix:tmpdir=/tmp",
            ])
            .stdout(Stdio::piped())
            .spawn()
            .expect("start private test bus");
        let mut bus = TestBus(child);
        let mut address = String::new();
        std::io::BufReader::new(bus.0.stdout.take().unwrap())
            .read_line(&mut address)
            .unwrap();
        let address = address.trim();
        let (sender, mut receiver) = mpsc::channel(16);
        let owner = zbus::connection::Builder::address(address)
            .unwrap()
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name(RUNTIME_NAME)
            .unwrap()
            .serve_at(OBJECT_PATH, RuntimeInterface(sender.clone()))
            .unwrap()
            .build()
            .await
            .unwrap();
        let client = zbus::connection::Builder::address(address)
            .unwrap()
            .build()
            .await
            .unwrap();
        let proxy = zbus::Proxy::new(&client, RUNTIME_NAME, OBJECT_PATH, RUNTIME_INTERFACE)
            .await
            .unwrap();
        for command in [
            Command::Status,
            Command::SetUnmount(true),
            Command::SetRestore(true),
            Command::SetPreload(PreloadSettings::default()),
            Command::Refresh,
            Command::ConfigurationChanged,
        ] {
            let expected = Status {
                unmount_before_sleep: true,
                restore_after_wake: true,
                preload: PreloadSettings::default(),
                sleep_status: Some("ready".into()),
            };
            let json = serde_json::to_string(&command).unwrap();
            let args = (json,);
            let request = proxy.call::<_, _, String>("Request", &args);
            let handle = async {
                let Event::Request(received, reply) = receiver.recv().await.unwrap() else {
                    panic!("expected request")
                };
                assert_eq!(received, command);
                reply.complete(Ok(expected.clone()));
                // A clone cannot send a second reply or replace the first result.
                reply.complete(Err("late result".into()));
            };
            let (response, ()) = tokio::join!(request, handle);
            assert_eq!(
                serde_json::from_str::<Status>(&response.unwrap()).unwrap(),
                expected
            );
        }
        let json = serde_json::to_string(&Command::SetUnmount(false)).unwrap();
        let args = (json,);
        let request = proxy.call::<_, _, String>("Request", &args);
        let handle = async {
            let Event::Request(_, reply) = receiver.recv().await.unwrap() else {
                panic!("expected request")
            };
            reply.complete(Err("configuration is read-only".into()));
        };
        let (response, ()) = tokio::join!(request, handle);
        assert!(
            response
                .unwrap_err()
                .to_string()
                .contains("configuration is read-only")
        );
        assert!(
            proxy
                .call::<_, _, String>("Request", &("unknown-command",))
                .await
                .is_err()
        );
        assert!(receiver.try_recv().is_err());
        // Another process cannot take ownership and start a second listener.
        assert!(
            zbus::connection::Builder::address(address)
                .unwrap()
                .allow_name_replacements(false)
                .replace_existing_names(false)
                .name(RUNTIME_NAME)
                .unwrap()
                .build()
                .await
                .is_err()
        );

        let settings = zbus::connection::Builder::address(address)
            .unwrap()
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name(SETTINGS_NAME)
            .unwrap()
            .serve_at(OBJECT_PATH, SettingsInterface(sender))
            .unwrap()
            .build()
            .await
            .unwrap();
        let focus = zbus::Proxy::new(&client, SETTINGS_NAME, OBJECT_PATH, SETTINGS_INTERFACE)
            .await
            .unwrap();
        focus.call::<_, _, ()>("Activate", &()).await.unwrap();
        assert!(matches!(receiver.recv().await, Some(Event::Activate)));
        assert!(
            zbus::connection::Builder::address(address)
                .unwrap()
                .allow_name_replacements(false)
                .replace_existing_names(false)
                .name(SETTINGS_NAME)
                .unwrap()
                .build()
                .await
                .is_err()
        );
        drop(settings);
        // Closing settings leaves the runtime service alive.
        let json = serde_json::to_string(&Command::Status).unwrap();
        let args = (json,);
        let request = proxy.call::<_, _, String>("Request", &args);
        let handle = async {
            let Event::Request(_, reply) = receiver.recv().await.unwrap() else {
                panic!("expected request")
            };
            reply.complete(Ok(Status::default()));
        };
        let (response, ()) = tokio::join!(request, handle);
        assert!(response.is_ok());
        owner.release_name(RUNTIME_NAME).await.unwrap();
        assert!(
            proxy
                .call::<_, _, String>("Request", &("\"Status\"",))
                .await
                .is_err()
        );
    }
}
