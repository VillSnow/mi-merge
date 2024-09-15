use std::{collections::HashSet, error::Error, future::Future};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::json;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, error::TryRecvError, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{common_types::Host, mi_models::WsMsg};

#[derive(Debug)]
pub enum ServerCxnError {
    ConnectError,
    SendToClosedServerError,
    HttpRequestError,
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
            ServerCxnError::HttpRequestError => {
                write!(f, "http request error")
            }
        }
    }
}

impl Error for ServerCxnError {}

impl From<reqwest::Error> for ServerCxnError {
    fn from(_value: reqwest::Error) -> Self {
        Self::HttpRequestError
    }
}

#[derive(Debug)]
enum ThrResource<T> {
    Online(JoinHandle<T>),
    Offline(T),
    Uninit,
}

impl<T> ThrResource<T> {
    fn into_online<F>(&mut self, f: impl FnOnce(T) -> F)
    where
        F: Future<Output = T> + Send + 'static,
        F::Output: Send + 'static,
    {
        let r = match std::mem::replace(self, Self::Uninit) {
            Self::Online(_) => panic!("invalid operation"),
            Self::Offline(r) => r,
            Self::Uninit => unreachable!(),
        };
        *self = Self::Online(tokio::spawn(f(r)));
    }
}

struct RecvThr {
    tx: UnboundedSender<WsMsg>,
    ws_rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl RecvThr {
    async fn run(mut self) -> UnboundedSender<WsMsg> {
        while let Some(Ok(m)) = self.ws_rx.next().await {
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

                    self.tx.send(m).expect("mpsc error");
                }
                Message::Ping(_) => {}
                m => debug!("{m:?}"),
            }
        }

        self.tx
    }
}

struct SendThr {
    rx: UnboundedReceiver<String>,
    pending: Option<String>,
    ws_tx: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    idle_tx: Option<oneshot::Sender<()>>,
}

impl SendThr {
    async fn run(mut self) -> (UnboundedReceiver<String>, Option<String>) {
        if let Some(m) = self.pending.take() {
            dbg!();
            if self.ws_tx.send(Message::Text(m.clone())).await.is_err() {
                return (self.rx, Some(m));
            }
        }

        loop {
            match self.rx.try_recv() {
                Ok(m) => {
                    if self.ws_tx.send(Message::Text(m.clone())).await.is_err() {
                        return (self.rx, Some(m));
                    };
                }
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    if let Some(idle_tx) = self.idle_tx.take() {
                        idle_tx.send(()).expect("channel error");
                    }
                    tokio::task::yield_now().await;
                    continue;
                }
            }
        }

        (self.rx, None)
    }
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
        let (ws_tx, ws_rx) = ws.split();
        let (idle_tx, idle_rs) = oneshot::channel::<()>();

        self.recv_thr.into_online(|tx| RecvThr { tx, ws_rx }.run());
        self.send_thr.into_online(|(rx, pending)| {
            SendThr {
                rx,
                pending,
                ws_tx,
                idle_tx: Some(idle_tx),
            }
            .run()
        });

        match idle_rs.await {
            Ok(_) => info!("the receive thread is idle"),
            Err(_) => warn!("the receive thread has downed before it becomes idle"),
        }

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

    pub fn connect_to_channel(&mut self, channel_id: &str) -> String {
        let cxn_channel_id = Uuid::new_v4().to_string();
        self.send(
            json!({
                "type": "connect",
                "body": {
                    "id": cxn_channel_id.clone(),
                    "channel": "channel",
                    "params": {
                        "channelId": channel_id
                    }
                }
            })
            .to_string(),
        );
        cxn_channel_id
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
