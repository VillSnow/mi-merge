use std::{collections::HashSet, error::Error, fs::File, io::BufReader, sync::Arc};

use fancy_regex::Regex;
use itertools::Itertools;
use serde_json::json;
use tokio::sync::{mpsc::UnboundedReceiver, RwLock};
use tracing::error;

use crate::{
    common_types::{
        BranchKey, ChannelChannel, Connection, Credential, DynNoteModel, Host, NoteModel,
    },
    merged_timeline::MergedTimeline,
    mi_models::Note,
    server_cxn::ServerCxn,
    server_note_repo::ServerNoteRepo,
    ws_msg_router::WsMsgRouter,
    ws_poller::WsPoller,
};

#[derive(Debug)]
pub struct AppModel {
    pub credentials: Vec<Credential>,
    pub merged_timeline: Arc<RwLock<MergedTimeline>>,

    branches: Vec<BranchKey>,
    branches_set: HashSet<BranchKey>,
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
            credentials: Default::default(),
            merged_timeline: Arc::new(RwLock::new(MergedTimeline::new())),
            branches: Vec::new(),
            branches_set: HashSet::new(),
        }
    }

    pub async fn connect_all(&mut self) -> Result<(), Box<dyn Error>> {
        self.credentials = serde_json::from_reader(BufReader::new(
            File::open("credentials.json").expect("TODO: handle error"),
        ))
        .map_err(|e| {
            error!("failed to parse credentials.json");
            e
        })?;
        let column: Vec<Connection> = serde_json::from_reader(BufReader::new(
            File::open("connections.json").expect("TODO: handle error"),
        ))
        .map_err(|e| {
            error!("failed to parse connections.json");
            e
        })?;

        for c in column.into_iter().filter(|x| !x.disable) {
            self.connect(c).await
        }
        self.merged_timeline.write().await.implicit_sort().await;

        Ok(())
    }

    pub async fn connect(&mut self, cxn_settings: Connection) {
        let (host, api_key) = {
            let credential = self
                .credentials
                .iter()
                .filter(|x| x.host == cxn_settings.host && x.user == cxn_settings.user)
                .next();
            let credential = credential.expect("missing credential");

            let host = Host::from(cxn_settings.host.clone());
            let api_key = credential.api_key.clone();
            (host, api_key)
        };

        let mut server_cxn = ServerCxn::new(host.clone(), api_key.clone());

        let channel_branches = cxn_settings
            .channels
            .clone()
            .into_iter()
            .map(|x| (x.channel, x.branches))
            .into_grouping_map()
            .fold(HashSet::new(), |mut acc, _k, v| {
                acc.extend(v.into_iter().map(BranchKey));
                acc
            });

        for b in cxn_settings.channels.iter().flat_map(|x| &x.branches) {
            self.insert_branch(BranchKey(b.clone()));
        }

        let mut router = WsMsgRouter::new();
        for (channel, branches) in &channel_branches {
            let id = match channel {
                ChannelChannel::HomeTimeline => server_cxn.connect_to_home(),
                ChannelChannel::LocalTimeline => server_cxn.connect_to_local(),
                ChannelChannel::Channel { channel_id } => {
                    server_cxn.connect_to_channel(&channel_id)
                }
            };
            router.extend(id, branches.iter().cloned());
        }

        server_cxn.spawn().await.expect("TODO: handle error");

        let mut repo = ServerNoteRepo::new();
        let receiver = repo.make_updated_note_receiver();

        let cxn = Arc::new(RwLock::new(server_cxn));
        let repo = Arc::new(RwLock::new(repo));

        let poller = WsPoller {
            repo: repo.clone(),
            cxn: cxn.clone(),
            router,
            host: host.clone(),
        };
        tokio::spawn(poller.poll());

        for (channel, branches) in &channel_branches {
            let notes = match channel {
                ChannelChannel::HomeTimeline => fetch_home_notes(&host, &api_key)
                    .await
                    .expect("TODO: handle error"),
                ChannelChannel::LocalTimeline => fetch_local_notes(&host, &api_key)
                    .await
                    .expect("TODO: handle error"),
                ChannelChannel::Channel { channel_id } => {
                    fetch_channel_notes(&host, &api_key, &channel_id)
                        .await
                        .expect("TODO: handle error")
                }
            };

            let mut repo = repo.write().await;
            for note in notes {
                repo.upsert(
                    NoteModel::from_mi_model(note, host.clone()),
                    branches.clone(),
                )
                .expect("TODO: handle error");
            }
        }

        let merger = TimelineMerger {
            merged_timeline: self.merged_timeline.clone(),
            host: host.clone(),
            receiver,
        };
        tokio::spawn(merger.merge());
    }

    pub fn branches(&self) -> Vec<BranchKey> {
        self.branches.clone()
    }

    pub fn insert_branch(&mut self, branch: BranchKey) {
        if self.branches_set.insert(branch.clone()) {
            self.branches.push(branch);
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

async fn fetch_channel_notes(
    host: &Host,
    api_key: &str,
    channel_id: &str,
) -> Result<Vec<Note>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://{}/api/channels/timeline",
            host.to_string()
        ))
        .json(&json!({ "channelId": channel_id, "i": api_key }))
        .send()
        .await?
        .error_for_status()?;

    let res = res.json().await?;
    Ok(res)
}
