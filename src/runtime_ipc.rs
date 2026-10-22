// SPDX-License-Identifier: MIT

//! The panel owns runtime state. Standalone windows only send requests.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cosmic::iced::{
    Subscription,
    futures::{SinkExt, StreamExt},
    stream,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use cosmic_ext_applet_mounter::model::{ConnectionId, Operation, PreloadSettings};

pub const RUNTIME_NAME: &str = "io.github.uutzinger.cosmic-ext-applet-mounter.Runtime";
const SETTINGS_NAME: &str = "io.github.uutzinger.cosmic-ext-applet-mounter.Settings";
const OBJECT_PATH: &str = "/io/github/uutzinger/CloudMounter";
const RUNTIME_INTERFACE: &str = "io.github.uutzinger.CloudMounter.Runtime";
const SETTINGS_INTERFACE: &str = "io.github.uutzinger.CloudMounter.Settings";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
    Status,
    SetUnmount(bool),
    SetRestore(bool),
    SetPreload(PreloadSettings),
    ConfigurationChanged,
    RunOperation(ConnectionId, Operation),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Status {
    pub unmount_before_sleep: bool,
    pub restore_after_wake: bool,
    pub preload: PreloadSettings,
    pub sleep_status: Option<String>,
    #[serde(default)]
    pub sleep_needs_attention: bool,
    #[serde(default)]
    pub operation_revision: u64,
    #[serde(default)]
    pub operation_completions: Vec<OperationCompletion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationCompletion {
    pub revision: u64,
    pub connection_id: ConnectionId,
    pub operation: Operation,
    pub notice: String,
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
            .map_err(|_| zbus::fdo::Error::Failed(crate::fl!("ui-runtime-ipc-applet-closed")))?;
        let status = tokio::time::timeout(Duration::from_secs(120), receiver)
            .await
            .map_err(|_| {
                zbus::fdo::Error::Failed(crate::fl!(
                    "ui-runtime-ipc-applet-request-timed-out-check-current-status"
                ))
            })?
            .map_err(|_| {
                zbus::fdo::Error::Failed(crate::fl!("ui-runtime-ipc-applet-closed-before-replying"))
            })?
            .map_err(zbus::fdo::Error::Failed)?;
        serde_json::to_string(&status).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }
}

struct SettingsInterface(mpsc::Sender<Event>);
#[zbus::interface(name = "io.github.uutzinger.CloudMounter.Settings")]
impl SettingsInterface {
    async fn activate(&self) -> zbus::fdo::Result<()> {
        self.0.send(Event::Activate).await.map_err(|_| {
            zbus::fdo::Error::Failed(crate::fl!("ui-runtime-ipc-settings-window-closed"))
        })
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
    .map_err(|_| crate::fl!("ui-runtime-ipc-applet-request-timed-out-check-current-status"))?
    .map_err(|e| {
        crate::fl!(
            "ui-runtime-ipc-could-not-contact-or-update-the-running",
            e = (e).to_string()
        )
    })
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
    if !settings {
        // Retry transient bus startup/disconnection failures with a fixed bound.
        // Competing panel instances queue on D-Bus instead of polling the owner.
        for attempt in 0..5 {
            let (sender, receiver) = mpsc::channel(16);
            let result = async {
                let connection = zbus::connection::Builder::session()?
                    .serve_at(OBJECT_PATH, RuntimeInterface(sender))?
                    .build()
                    .await?;
                serve_runtime_connection(connection, receiver, &mut output).await
            }
            .await;
            if let Err(error) = result {
                if output
                    .send(Event::Error(crate::fl!(
                        "ui-runtime-ipc-window-communication-unavailable-error",
                        error = error.to_string()
                    )))
                    .await
                    .is_err()
                {
                    return;
                }
            } else {
                return;
            }
            if attempt < 4 {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        std::future::pending::<()>().await;
    }
    let (sender, mut receiver) = mpsc::channel(16);
    let result = async {
        let builder = zbus::connection::Builder::session()?;
        builder
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name(SETTINGS_NAME)?
            .serve_at(OBJECT_PATH, SettingsInterface(sender))?
            .build()
            .await
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
            if activate_settings().await {
                let _ = output.send(Event::DuplicateWindow).await;
            } else {
                let _ = output
                    .send(Event::Error(crate::fl!(
                        "ui-runtime-ipc-window-communication-unavailable-error",
                        error = (error).to_string()
                    )))
                    .await;
            }
            std::future::pending::<()>().await;
        }
    }
}

// Register signal listeners before requesting the name, avoiding a handoff race.
// Empty request flags queue without replacing or allowing replacement of an owner.
async fn serve_runtime_connection(
    connection: zbus::Connection,
    mut receiver: mpsc::Receiver<Event>,
    output: &mut cosmic::iced::futures::channel::mpsc::Sender<Event>,
) -> zbus::Result<()> {
    use zbus::fdo::{DBusProxy, RequestNameReply};
    let bus = DBusProxy::new(&connection).await?;
    let mut acquired = bus.receive_name_acquired().await?;
    let mut lost = bus.receive_name_lost().await?;
    let reply = connection
        .request_name_with_flags(RUNTIME_NAME, enumflags2::BitFlags::empty())
        .await?;
    let mut owner = matches!(
        reply,
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner
    );
    if output
        .send(if owner {
            Event::Ready
        } else {
            Event::ClientReady
        })
        .await
        .is_err()
    {
        return Ok(());
    }
    loop {
        tokio::select! {
            signal = acquired.next() => {
                let signal = signal.ok_or(zbus::Error::Failure(crate::fl!("runtime-ipc-connection-closed")))?;
                if signal.args()?.name.as_str() == RUNTIME_NAME && !owner {
                    owner = true;
                    if output.send(Event::Ready).await.is_err() { return Ok(()); }
                }
            }
            signal = lost.next() => {
                let signal = signal.ok_or(zbus::Error::Failure(crate::fl!("runtime-ipc-connection-closed")))?;
                if signal.args()?.name.as_str() == RUNTIME_NAME {
                    return Err(zbus::Error::NameTaken);
                }
            }
            event = receiver.recv() => {
                let Some(event) = event else { return Ok(()); };
                if output.send(event).await.is_err() { return Ok(()); }
            }
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

    #[test]
    fn status_accepts_payloads_without_operation_completion_fields() {
        let mut value = serde_json::to_value(Status::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("operation_revision");
        object.remove("operation_completions");
        object.remove("sleep_needs_attention");

        let status: Status = serde_json::from_value(value).unwrap();

        assert_eq!(status.operation_revision, 0);
        assert!(status.operation_completions.is_empty());
        assert!(!status.sleep_needs_attention);
    }

    #[tokio::test]
    async fn runtime_ipc_queued_panels_take_over_without_replacing_the_owner() {
        tokio::time::timeout(Duration::from_secs(10), async {
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
                .unwrap();
            let mut bus = TestBus(child);
            let mut address = String::new();
            std::io::BufReader::new(bus.0.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            let address = address.trim();
            let original = zbus::connection::Builder::address(address)
                .unwrap()
                .allow_name_replacements(false)
                .replace_existing_names(false)
                .name(RUNTIME_NAME)
                .unwrap()
                .build()
                .await
                .unwrap();
            let observer = zbus::connection::Builder::address(address)
                .unwrap()
                .build()
                .await
                .unwrap();
            let bus_proxy = zbus::fdo::DBusProxy::new(&observer).await.unwrap();
            let original_id = original.unique_name().unwrap().as_str().to_owned();
            let mut panels = Vec::new();
            for _ in 0..2 {
                let (sender, receiver) = mpsc::channel(16);
                let connection = zbus::connection::Builder::address(address)
                    .unwrap()
                    .serve_at(OBJECT_PATH, RuntimeInterface(sender))
                    .unwrap()
                    .build()
                    .await
                    .unwrap();
                let id = connection.unique_name().unwrap().as_str().to_owned();
                let (mut output, mut events) = cosmic::iced::futures::channel::mpsc::channel(16);
                let task = tokio::spawn(async move {
                    serve_runtime_connection(connection, receiver, &mut output).await
                });
                assert!(matches!(events.next().await, Some(Event::ClientReady)));
                assert_eq!(
                    bus_proxy
                        .get_name_owner(RUNTIME_NAME.try_into().unwrap())
                        .await
                        .unwrap()
                        .as_str(),
                    original_id
                );
                panels.push((id, events, task));
            }
            original.close().await.unwrap();
            let (first_id, first_events, first_task) = &mut panels[0];
            assert!(matches!(first_events.next().await, Some(Event::Ready)));
            assert_eq!(
                bus_proxy
                    .get_name_owner(RUNTIME_NAME.try_into().unwrap())
                    .await
                    .unwrap()
                    .as_str(),
                first_id
            );
            let proxy = zbus::Proxy::new(&observer, RUNTIME_NAME, OBJECT_PATH, RUNTIME_INTERFACE)
                .await
                .unwrap();
            let args = (serde_json::to_string(&Command::ConfigurationChanged).unwrap(),);
            let request = proxy.call::<_, _, String>("Request", &args);
            let handle = async {
                let Some(Event::Request(Command::ConfigurationChanged, reply)) =
                    first_events.next().await
                else {
                    panic!("expected configuration reload");
                };
                reply.complete(Ok(Status::default()));
            };
            let (response, ()) = tokio::join!(request, handle);
            assert!(response.is_ok());
            first_task.abort();
            let _ = first_task.await;
            let (second_id, second_events, second_task) = &mut panels[1];
            assert!(matches!(second_events.next().await, Some(Event::Ready)));
            assert_eq!(
                bus_proxy
                    .get_name_owner(RUNTIME_NAME.try_into().unwrap())
                    .await
                    .unwrap()
                    .as_str(),
                second_id
            );
            second_task.abort();
            let _ = second_task.await;
        })
        .await
        .expect("bounded owner handoff");
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
            Command::ConfigurationChanged,
            Command::RunOperation(
                ConnectionId::from_uuid(
                    uuid::Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").unwrap(),
                ),
                Operation::SyncNow,
            ),
        ] {
            let expected = Status {
                unmount_before_sleep: true,
                restore_after_wake: true,
                preload: PreloadSettings::default(),
                sleep_status: Some("ready".into()),
                ..Status::default()
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
