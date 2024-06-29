use std::{
    error::Error,
    future::Future,
    mem::{forget, MaybeUninit},
};

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{
    sync::mpsc::{self, error::TryRecvError, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{common_types::Host, entries::WsMsg};

#[derive(Debug)]
pub enum ServerCxnError {
    ConnectError,
    SendToClosedServerError,
}

impl std::fmt::Display for ServerCxnError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServerCxnError::ConnectError => {
                write!(f, "failed to connect to the server")
            }
            ServerCxnError::SendToClosedServerError => {
                write!(f, "tried to send to eventually disconnected server")
            }
        }
    }
}

impl Error for ServerCxnError {}

#[derive(Debug)]
enum ThrResource<T> {
    Online(JoinHandle<T>),
    Offline(T),
}

#[derive(Debug)]
pub struct ServerCxn {
    host: Host,
    api_key: String,

    outlet: UnboundedReceiver<WsMsg>,
    inlet: UnboundedSender<String>,

    recv_thr: ThrResource<UnboundedSender<WsMsg>>,
    send_thr: ThrResource<(UnboundedReceiver<String>, Option<String>)>,

    home_timeline_id: Option<String>,
    local_timeline_id: Option<String>,
}

impl<T> ThrResource<T> {
    fn into_online<F>(&mut self, f: impl FnOnce(T) -> F)
    where
        F: Future<Output = T> + Send + 'static,
        F::Output: Send + 'static,
    {
        if !matches!(self, ThrResource::Offline(_)) {
            panic!("invalid operation");
        }
        unsafe {
            let value = std::mem::replace(self, MaybeUninit::uninit().assume_init());

            match value {
                ThrResource::Offline(r) => {
                    let uninit = std::mem::replace(self, Self::Online(tokio::spawn(f(r))));
                    forget(uninit);
                }
                _ => unreachable!(),
            }
        }
    }
}

impl ServerCxn {
    pub fn new(host: Host, api_key: String) -> Self {
        let (outlet_tx, outlet) = mpsc::unbounded_channel::<WsMsg>();
        let (inlet, inlet_rx) = mpsc::unbounded_channel::<String>();
        Self {
            host,
            api_key,

            outlet,
            inlet,
            recv_thr: ThrResource::Offline(outlet_tx),
            send_thr: ThrResource::Offline((inlet_rx, None)),

            home_timeline_id: None,
            local_timeline_id: None,
        }
    }

    pub async fn spawn(&mut self) -> Result<(), ServerCxnError> {
        info!("connecting to {}", self.host);
        let req = format!("wss://{}/stream?i={}", self.host, self.api_key);
        let (ws, _res) = connect_async(req)
            .await
            .map_err(|_| ServerCxnError::ConnectError)?;
        let (mut ws_tx, mut ws_rs) = ws.split();

        self.recv_thr.into_online(|outlet_rx| async move {
            while let Some(Ok(m)) = ws_rs.next().await {
                match m {
                    Message::Text(m) => {
                        let m = match serde_json::from_str::<serde_json::Value>(&m) {
                            Ok(m) => m,
                            Err(e) => {
                                error!("{e:?}");
                                continue;
                            }
                        };
                        let m = match serde_json::from_value::<WsMsg>(m) {
                            Ok(m) => m,
                            Err(e) => {
                                info!("{e:?}");
                                continue;
                            }
                        };
                        debug!("{m:?}");

                        outlet_rx.send(m).expect("mpsc error");
                    }
                    Message::Ping(_) => {}
                    m => debug!("{m:?}"),
                }
            }

            outlet_rx
        });

        self.send_thr.into_online(|(mut inlet_tx, m)| async move {
            if let Some(m) = m {
                dbg!();
                if ws_tx.send(Message::Text(m.clone())).await.is_err() {
                    return (inlet_tx, Some(m));
                }
            }
            while let Some(m) = inlet_tx.recv().await {
                if ws_tx.send(Message::Text(m.clone())).await.is_err() {
                    return (inlet_tx, Some(m));
                };
            }
            (inlet_tx, None)
        });

        Ok(())
    }

    pub fn connect_to_home(&mut self) -> String {
        let home_timeline_id = Uuid::new_v4().to_string();
        self.send(
            json!({
                "type": "connect",
                "body": {
                    "id": home_timeline_id.clone(),
                    "channel": "homeTimeline",
                    "params": {}
                }
            })
            .to_string(),
        );
        self.home_timeline_id = Some(home_timeline_id.clone());
        home_timeline_id
    }

    pub fn connect_to_local(&mut self) -> String {
        let local_timeline_id = Uuid::new_v4().to_string();
        self.send(
            json!({
                "type": "connect",
                "body": {
                    "id": local_timeline_id.clone(),
                    "channel": "localTimeline",
                    "params": {}
                }
            })
            .to_string(),
        );
        self.local_timeline_id = Some(local_timeline_id.clone());
        local_timeline_id
    }

    pub fn subscribe_note(&mut self, note_id: &str) {
        self.send(
            json!({
                "type": "subNote",
                "body": {
                    "id": note_id
                }
            })
            .to_string(),
        );
    }

    pub fn send(&self, message: String) {
        self.inlet.send(message).unwrap()
    }

    pub async fn recv(&mut self) -> Option<WsMsg> {
        self.outlet.recv().await
    }

    pub fn try_recv(&mut self) -> Result<WsMsg, TryRecvError> {
        self.outlet.try_recv()
    }
}
