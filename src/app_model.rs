use std::{collections::HashSet, error::Error, fs::File, io::BufReader, sync::Arc, time::Duration};

use fancy_regex::Regex;
use serde_json::json;
use tokio::sync::{mpsc::UnboundedReceiver, RwLock};

use crate::{
    common_types::{BranchKey, BranchTimeline, Credential, DynNoteModel, Host, NoteModel},
    merged_timeline::MergedTimeline,
    mi_models::Note,
    server_cxn::ServerCxn,
    server_note_repo::ServerNoteRepo,
    ws_poller::WsPoller,
};

#[derive(Debug)]
pub struct AppModel {
    pub merged_timeline: Arc<RwLock<MergedTimeline>>,

    pub branches: Vec<BranchKey>,
}

#[derive(Debug)]
pub struct TimelineMerger {
    merged_timeline: Arc<RwLock<MergedTimeline>>,
    host: Host,
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
        self.branches.push(BranchKey {
            host: host.clone(),
            timeline: BranchTimeline::Home,
        });

        let local_timeline_id = cxn.connect_to_local();
        self.branches.push(BranchKey {
            host: host.clone(),
            timeline: BranchTimeline::Local,
        });

        cxn.spawn().await.expect("TODO: handle error");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut repo = ServerNoteRepo::new();
        let receiver = repo.make_updated_note_receiver();

        let cxn = Arc::new(RwLock::new(cxn));
        let repo = Arc::new(RwLock::new(repo));

        let poller = WsPoller {
            repo: repo.clone(),
            cxn: cxn.clone(),
            host: host.clone(),
            home_timeline_id,
            local_timeline_id,
        };
        tokio::spawn(poller.poll());

        let merger = TimelineMerger {
            merged_timeline: self.merged_timeline.clone(),
            host: host.clone(),
            receiver,
        };
        tokio::spawn(merger.merge());

        match fetch_home_notes(host, api_key).await {
            Ok(notes) => {
                let mut cxn = cxn.write().await;
                let mut repo = repo.write().await;
                let branches = HashSet::from([BranchKey {
                    host: host.clone(),
                    timeline: BranchTimeline::Home,
                }]);

                for note in notes.into_iter().rev() {
                    let note_id = note.id.clone();
                    let model = NoteModel::from_mi_model(note, host.clone());
                    repo.upsert(model, branches.clone())
                        .expect("TODO: handle error");
                    cxn.subscribe_note(&note_id);
                }
            }
            Err(e) => {
                tracing::error!("failed to fetch notes: {e}");
            }
        }
        match fetch_local_notes(host, api_key).await {
            Ok(notes) => {
                let mut tl = repo.write().await;
                let branches = HashSet::from([BranchKey {
                    host: host.clone(),
                    timeline: BranchTimeline::Local,
                }]);

                for note in notes.into_iter().rev() {
                    let model = NoteModel::from_mi_model(note, host.clone());
                    tl.upsert(model, branches.clone())
                        .expect("TODO: handle error");
                }
            }
            Err(e) => {
                tracing::error!("failed to fetch notes: {e}");
            }
        }
    }
}

impl TimelineMerger {
    async fn merge(mut self) {
        while let Some(mut note) = self.receiver.recv().await {
            for (r, _) in &mut note.reactions {
                if let Some(qualified) = self.qualify_reaction(r) {
                    *r = qualified;
                }
            }

            self.merged_timeline
                .write()
                .await
                .upsert(note)
                .await
                .expect("TODO: handle error");
        }
    }

    fn qualify_reaction(&self, reaction_name: &str) -> Option<String> {
        let re = Regex::new("^:(.*)@(.*):$").unwrap();
        match re.captures(&reaction_name).expect("regex error") {
            Some(captures) => {
                if captures.get(2).unwrap().as_str() == "." {
                    return Some(format!(
                        ":{}@{}:",
                        captures.get(1).unwrap().as_str(),
                        self.host
                    ));
                } else {
                    return None;
                }
            }
            None => None,
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
