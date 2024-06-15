use std::{
    error::Error,
    future::Future,
    mem::{forget, MaybeUninit},
    sync::Arc,
};

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{common_types::Host, entries::WsMsg, subject::SubjectMut};

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

    received_messages: Arc<Mutex<SubjectMut<WsMsg>>>,

    hybrid_timeline_id: Option<String>,
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

            received_messages: Arc::new(Mutex::new(SubjectMut::new())),

            hybrid_timeline_id: None,
        }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub async fn spawn(&mut self) -> Result<(), ServerCxnError> {
        info!("connecting to {}", self.host);
        let req = format!("wss://{}/stream?i={}", self.host, self.api_key);
        let (ws, _res) = connect_async(req)
            .await
            .map_err(|_| ServerCxnError::ConnectError)?;
        let (mut ws_tx, mut ws_rs) = ws.split();

        let received_messages = self.received_messages.clone();
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

                        received_messages.lock().await.next(m).await;
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

    pub fn connect_to_hybrid(&mut self) {
        let hybrid_timeline_id = Uuid::new_v4().to_string();
        self.send(
            json!({
                "type": "connect",
                "body": {
                    "id": hybrid_timeline_id.clone(),
                    "channel": "hybridTimeline",
                    "params": {}
                }
            })
            .to_string(),
        );
        self.hybrid_timeline_id = Some(hybrid_timeline_id);
    }

    pub fn send(&self, message: String) {
        self.inlet.send(message).unwrap()
    }

    pub async fn subscribe<F, R>(&mut self, mut callback: F)
    where
        F: (FnMut(WsMsg) -> R) + Send + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.received_messages
            .lock()
            .await
            .subscribe(callback)
            .await;
    }
}
