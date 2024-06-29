use std::{collections::HashSet, sync::Arc};

use tokio::sync::{mpsc::error::TryRecvError, RwLock};
use tracing::warn;

use crate::{
    common_types::{BranchKey, BranchTimeline, Host, NoteModel},
    mi_models::{NoteUpdatedBody, WsMsg, WsMsgChannelBody},
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
                WsMsg::Channel(WsMsgChannelBody::Note { id: ch_id, body }) => {
                    let note_id = body.id.clone();

                    let branch = if ch_id == self.home_timeline_id {
                        BranchKey {
                            host: self.host.clone(),
                            timeline: BranchTimeline::Home,
                        }
                    } else if ch_id == self.local_timeline_id {
                        BranchKey {
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
                            NoteModel::from_mi_model(body, self.host.clone()),
                            HashSet::from([branch]),
                        )
                        .expect("TODO: handle error");

                    self.cxn.write().await.subscribe_note(&note_id);
                }
                WsMsg::NoteUpdated(NoteUpdatedBody::NoteUpdatedBodyReacted {
                    id: note_id,
                    body,
                }) => {
                    self.timeline
                        .write()
                        .await
                        .incr_reaction(&note_id, &body.reaction);
                }
            }
        }
    }
}
