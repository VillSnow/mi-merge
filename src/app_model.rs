use std::{collections::HashSet, error::Error, fs::File, io::BufReader, sync::Arc};

use serde_json::json;
use tokio::sync::{
    mpsc::{error::TryRecvError, UnboundedReceiver},
    RwLock,
};
use tracing::warn;

use crate::{
    common_types::{Branch, BranchTimeline, Credential, DynNoteModel, Host, NoteModel},
    entries::{Note, WsMsg, WsMsgChannelBody},
    merged_timeline::MergedTimeline,
    server_cxn::ServerCxn,
    server_note_repo::ServerNoteRepo,
};

#[derive(Debug)]
pub struct AppModel {
    pub merged_timeline: Arc<RwLock<MergedTimeline>>,

    pub branches: Vec<Branch>,
}

#[derive(Debug)]
pub struct WsPoller {
    timeline: Arc<RwLock<ServerNoteRepo>>,
    cxn: Arc<RwLock<ServerCxn>>,
    host: Host,
    home_timeline_id: String,
    local_timeline_id: String,
}

#[derive(Debug)]
pub struct TimelineMerger {
    merged_timeline: Arc<RwLock<MergedTimeline>>,
    receiver: UnboundedReceiver<DynNoteModel>,
}

impl AppModel {
    pub fn new() -> Self {
        Self {
            merged_timeline: Arc::new(RwLock::new(MergedTimeline::new())),
            branches: Vec::new(),
        }
    }

    pub async fn connect_all(&mut self) -> Result<(), Box<dyn Error>> {
        let credentials: Vec<Credential> = serde_json::from_reader(BufReader::new(
            File::open("credentials.json").expect("TODO: handle error"),
        ))?;
        for c in credentials.into_iter().filter(|x| !x.disable) {
            self.connect(&c.host.into(), &c.api_key).await
        }

        Ok(())
    }

    pub async fn connect(&mut self, host: &Host, api_key: &str) {
        let mut cxn = ServerCxn::new(host.clone(), api_key.to_owned());

        let home_timeline_id = cxn.connect_to_home();
        self.branches.push(Branch {
            host: host.clone(),
            timeline: BranchTimeline::Home,
        });

        let local_timeline_id = cxn.connect_to_local();
        self.branches.push(Branch {
            host: host.clone(),
            timeline: BranchTimeline::Local,
        });

        cxn.spawn().await.expect("TODO: handle error");

        let mut timeline = ServerNoteRepo::new();
        let receiver = timeline.make_updated_note_receiver();

        let cxn = Arc::new(RwLock::new(cxn));
        let timeline = Arc::new(RwLock::new(timeline));

        let poller = WsPoller {
            timeline: timeline.clone(),
            cxn: cxn.clone(),
            host: host.clone(),
            home_timeline_id,
            local_timeline_id,
        };
        tokio::spawn(poller.poll());

        let merger = TimelineMerger {
            merged_timeline: self.merged_timeline.clone(),
            receiver,
        };
        tokio::spawn(merger.merge());

        match fetch_home_notes(host, api_key).await {
            Ok(notes) => {
                let mut tl = self.merged_timeline.write().await;
                let branch = Branch {
                    host: host.clone(),
                    timeline: BranchTimeline::Home,
                };

                for note in notes.into_iter().rev() {
                    let mut dyn_model = DynNoteModel::from_ws_entity(note, host.clone());
                    dyn_model.branches.extend([branch.clone()]);
                    tl.upsert(dyn_model).await.expect("TODO: handle error");
                }
            }
            Err(e) => {
                tracing::error!("failed to fetch notes: {e}");
            }
        }
        match fetch_local_notes(host, api_key).await {
            Ok(notes) => {
                let mut tl = self.merged_timeline.write().await;
                let branch = Branch {
                    host: host.clone(),
                    timeline: BranchTimeline::Local,
                };

                for note in notes.into_iter().rev() {
                    let mut dyn_model = DynNoteModel::from_ws_entity(note, host.clone());
                    dyn_model.branches.extend([branch.clone()]);
                    tl.upsert(dyn_model).await.expect("TODO: handle error");
                }
            }
            Err(e) => {
                tracing::error!("failed to fetch notes: {e}");
            }
        }
    }
}

impl WsPoller {
    async fn poll(self) {
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

impl TimelineMerger {
    async fn merge(mut self) {
        while let Some(note) = self.receiver.recv().await {
            self.merged_timeline
                .write()
                .await
                .upsert(note)
                .await
                .expect("TODO: handle error");
        }
    }
}

async fn fetch_home_notes(host: &Host, api_key: &str) -> Result<Vec<Note>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("https://{}/api/notes/timeline", host.to_string()))
        .json(&json!({ "i": api_key }))
        .send()
        .await?
        .error_for_status()?;

    let res = res.json().await?;
    Ok(res)
}

async fn fetch_local_notes(host: &Host, api_key: &str) -> Result<Vec<Note>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://{}/api/notes/local-timeline",
            host.to_string()
        ))
        .json(&json!({ "i": api_key }))
        .send()
        .await?
        .error_for_status()?;

    let res = res.json().await?;
    Ok(res)
}
