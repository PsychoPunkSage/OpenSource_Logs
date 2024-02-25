use std::{fmt, path::Path, process::Stdio};

use anyhow::{anyhow, Context, Result};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use pcap_file::pcap::PcapPacket;
use tokio::{
    io::{AsyncReadExt, BufReader},
    process::{Child, Command},
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use zbus::export::futures_util::StreamExt;
use zbus::zvariant;

use crate::{timestamp::Timestamp, RUNTIME};

static IS_RUNNING_IN_FLATPAK: Lazy<bool> = Lazy::new(|| Path::new("/.flatpak-info").exists());

pub struct Event {
    pub message: zbus::Message,
    pub timestamp: Timestamp,
}

impl Event {
    pub fn from_packet(packet: PcapPacket<'_>) -> Result<Self> {
        Self::from_bytes(packet.data.into_owned(), packet.timestamp.into())
    }

    pub fn from_bytes(bytes: Vec<u8>, timestamp: Timestamp) -> Result<Self> {
        let ctx = zvariant::serialized::Context::new_dbus(0);
        let data =
            zvariant::serialized::Data::new_borrowed_fds(bytes, ctx, [] as [zvariant::Fd<'_>; 0]);
        let message =
            unsafe { zbus::Message::from_bytes(data).context("Failed to parse message")? };
        Ok(Self { message, timestamp })
    }
}

#[derive(Debug)]
pub struct Cancelled(String);

impl fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Cancelled {
    pub fn new(message: &str) -> Self {
        Self(format!("Cancelled monitoring: {}", message))
    }
}

#[derive(Debug)]
pub struct Monitor(Inner);

#[derive(Debug)]
enum Inner {
    System {
        command: Command,
        child: Option<Child>,
        tokio_handle: Option<JoinHandle<()>>,
        handle: Option<glib::JoinHandle<()>>,
    },
    Session {
        tokio_handle: Option<JoinHandle<()>>,
        handle: Option<glib::JoinHandle<()>>,
    },
}

impl Drop for Monitor {
    fn drop(&mut self) {
        match &mut self.0 {
            Inner::System {
                child,
                tokio_handle,
                handle,
                ..
            } => {
                if let Some(mut child) = child.take() {
                    glib::spawn_future(async move {
                        if let Err(err) = child.kill().await {
                            tracing::warn!("Failed to kill child process: {:?}", err)
                        } else {
                            tracing::debug!("Killed child process")
                        }
                    });
                }

                if let Some(tokio_handle) = tokio_handle.take() {
                    tokio_handle.abort();
                }

                if let Some(handle) = handle.take() {
                    handle.abort();
                }
            }
            Inner::Session {
                tokio_handle,
                handle,
                ..
            } => {
                if let Some(tokio_handle) = tokio_handle.take() {
                    tokio_handle.abort();
                }

                if let Some(handle) = handle.take() {
                    handle.abort();
                }
            }
        }
    }
}

impl Monitor {
    pub fn system() -> Self {
        let mut command = dbus_monitor_command();
        command
            .stdout(Stdio::piped())
            .arg("--binary")
            .arg("--system");

        Self(Inner::System {
            command,
            child: None,
            tokio_handle: None,
            handle: None,
        })
    }

    pub fn session() -> Self {
        Self(Inner::Session {
            tokio_handle: None,
            handle: None,
        })
    }

    pub fn address(address: zbus::Address) -> Self {
        let mut command = dbus_monitor_command();
        command
            .stdout(Stdio::piped())
            .arg("--binary")
            .arg("--address")
            .arg(address.to_string());

        Self(Inner::System {
            command,
            child: None,
            tokio_handle: None,
            handle: None,
        })
    }

    pub async fn start(&mut self, message_cb: impl Fn(Event) + 'static) -> Result<()> {
        match &mut self.0 {
            Inner::System {
                command,
                child,
                tokio_handle,
                handle,
            } => {
                let enter_guard = RUNTIME.enter();
                let mut spawned_child = command.spawn().context("Failed to spawn command")?;
                drop(enter_guard);

                let stdout = spawned_child.stdout.take().expect("Child must have stdout");

                let (tx, mut rx) = mpsc::channel(10);

                let (first_read_tx, first_read_rx) = oneshot::channel();
                let mut first_read_tx = Some(first_read_tx);

                tokio_handle.replace(RUNTIME.spawn(async move {
                    let mut reader = BufReader::new(stdout);

                    loop {
                        // Initialize space for the header
                        let mut message_buf = vec![0u8; 16];

                        // Read the header
                        let ret = reader.read_exact(&mut message_buf).await;

                        if let Some(first_msg_tx) = first_read_tx.take() {
                            let _ = first_msg_tx.send(());
                        }

                        if let Err(err) = ret {
                            tracing::warn!("Failed to read header bytes: {:?}", err);
                            break;
                        }

                        let timestamp = Timestamp::now();

                        let bytes_needed = match gio::DBusMessage::bytes_needed(&message_buf) {
                            Ok(n_needed) => n_needed,
                            Err(err) => {
                                tracing::warn!("Failed to get bytes needed: {:?}", err);
                                break;
                            }
                        };
                        assert!(bytes_needed >= 16);

                        // Make space for the body
                        message_buf.resize(bytes_needed as usize, 0);

                        // Read the body
                        if let Err(err) = reader.read_exact(&mut message_buf[16..]).await {
                            tracing::warn!("Failed to read body bytes: {:?}", err);
                            break;
                        }

                        match Event::from_bytes(message_buf, timestamp) {
                            Ok(event) => {
                                let _ = tx.send(event).await;
                            }
                            Err(err) => {
                                tracing::warn!("Failed to create event from bytes: {:?}", err);
                            }
                        }
                    }
                }));

                // Wait for the first read before we check for child status to ensure that the
                // child already exited on error.
                first_read_rx.await.expect("rx unexpectedly closed");

                if let Some(exit_status) =
                    spawned_child.try_wait().context("Failed to wait child")?
                {
                    debug_assert!(!exit_status.success(), "child must only exit on error");

                    let err = anyhow!(
                        "Child exited with status `{:?}` and code `{:?}`",
                        exit_status,
                        exit_status.code()
                    );
                    match exit_status.code() {
                        Some(126) => {
                            return Err(err.context(Cancelled::new(
                                "User dismissed polkit authorization dialog",
                            )))
                        }
                        _ => return Err(err),
                    }
                } else {
                    child.replace(spawned_child);
                }

                handle.replace(glib::spawn_future_local(async move {
                    while let Some(event) = rx.recv().await {
                        message_cb(event);
                    }
                }));
            }
            Inner::Session {
                tokio_handle,
                handle,
            } => {
                let cnx = RUNTIME
                    .spawn(async {
                        let cnx = zbus::Connection::session().await?;
                        let proxy = zbus::fdo::MonitoringProxy::new(&cnx).await?;
                        proxy.become_monitor(&[], 0).await?;
                        zbus::Result::Ok(cnx)
                    })
                    .await
                    .context("Failed to spawn on runtime")?
                    .context("Failed to setup monitoring")?;

                let (tx, mut rx) = mpsc::channel(10);

                tokio_handle.replace(RUNTIME.spawn(async move {
                    let mut stream = zbus::MessageStream::from(cnx);

                    while let Some(res) = stream.next().await {
                        match res {
                            Ok(message) => {
                                let _ = tx
                                    .send(Event {
                                        message,
                                        timestamp: Timestamp::now(),
                                    })
                                    .await;
                            }
                            Err(err) => tracing::warn!("Failed to receive message: {:?}", err),
                        }
                    }
                }));

                handle.replace(glib::spawn_future_local(async move {
                    while let Some(event) = rx.recv().await {
                        message_cb(event);
                    }
                }));
            }
        }

        Ok(())
    }
}

fn dbus_monitor_command() -> Command {
    let mut command = if *IS_RUNNING_IN_FLATPAK {
        Command::new("flatpak-spawn")
    } else {
        Command::new("pkexec")
    };

    if *IS_RUNNING_IN_FLATPAK {
        command.arg("--host").arg("pkexec");
    }

    command.arg("dbus-monitor");

    command
}
