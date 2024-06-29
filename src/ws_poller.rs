use std::{collections::HashSet, sync::Arc};

use tokio::sync::{mpsc::error::TryRecvError, RwLock};
use tracing::warn;

use crate::{
    common_types::{Branch, BranchTimeline, Host, NoteModel},
    entries::{WsMsg, WsMsgChannelBody},
    server_cxn::ServerCxn,
    server_note_repo::ServerNoteRepo,
};

#[derive(Debug)]
pub struct WsPoller {
    pub timeline: Arc<RwLock<ServerNoteRepo>>,
    pub cxn: Arc<RwLock<ServerCxn>>,
    pub host: Host,
    pub home_timeline_id: String,
    pub local_timeline_id: String,
}

impl WsPoller {
    pub async fn poll(self) {
        loop {
            let m = match self.cxn.write().await.try_recv() {
                Ok(m) => m,
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    tokio::task::yield_now().await;
                    continue;
                }
            };

            match m {
                WsMsg::Channel(WsMsgChannelBody::Note { id, body }) => {
                    let branch = if id == self.home_timeline_id {
                        Branch {
                            host: self.host.clone(),
                            timeline: BranchTimeline::Home,
                        }
                    } else if id == self.local_timeline_id {
                        Branch {
                            host: self.host.clone(),
                            timeline: BranchTimeline::Local,
                        }
                    } else {
                        warn!("unknown connection id");
                        return;
                    };

                    self.timeline
                        .write()
                        .await
                        .upsert(
                            NoteModel::from_ws_model(body, self.host.clone()),
                            HashSet::from([branch]),
                        )
                        .await
                        .expect("TODO: handle error");
                }
            }
        }
    }
}
