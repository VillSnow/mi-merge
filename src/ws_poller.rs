use std::sync::Arc;

use tokio::sync::{mpsc::error::TryRecvError, RwLock};

use crate::{
    common_types::{Host, NoteModel},
    mi_models::{NoteUpdatedBody, WsMsg, WsMsgChannelBody},
    server_cxn::ServerCxn,
    server_note_repo::ServerNoteRepo,
    ws_msg_router::WsMsgRouter,
};

#[derive(Debug)]
pub struct WsPoller {
    pub repo: Arc<RwLock<ServerNoteRepo>>,
    pub cxn: Arc<RwLock<ServerCxn>>,
    pub router: WsMsgRouter,
    pub host: Host,
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

                    self.repo
                        .write()
                        .await
                        .upsert(
                            NoteModel::from_mi_model(body, self.host.clone()),
                            self.router.solve_branches(&ch_id),
                        )
                        .expect("TODO: handle error");

                    self.cxn.write().await.subscribe_note(&note_id);
                }
                WsMsg::NoteUpdated(NoteUpdatedBody::NoteUpdatedBodyReacted {
                    id: note_id,
                    body,
                }) => {
                    self.repo
                        .write()
                        .await
                        .incr_reaction(&note_id, &body.reaction);
                }
            }
        }
    }
}
