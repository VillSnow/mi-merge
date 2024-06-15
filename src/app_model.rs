use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::BufReader,
    sync::Arc,
};

use serde_json::json;
use tokio::sync::RwLock;

use crate::{
    common_types::{Credential, Host},
    entries::{Note, WsMsg, WsMsgChannelBody},
    merged_timeline::MergedTimeline,
    server_cxn::ServerCxn,
};

#[derive(Debug)]
pub struct AppModel {
    pub merged_timeline: Arc<RwLock<MergedTimeline>>,
    pub server_cxn_store: HashMap<Host, Arc<RwLock<ServerCxn>>>,
}

impl AppModel {
    pub fn new() -> Self {
        Self {
            merged_timeline: Arc::new(RwLock::new(MergedTimeline::new())),
            server_cxn_store: Default::default(),
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
        assert!(!self.server_cxn_store.contains_key(host));

        let mut cxn = ServerCxn::new(host.clone(), api_key.to_owned());
        cxn.connect_to_hybrid();

        let tl = self.merged_timeline.clone();
        let host1 = host.clone();

        let f = |tl: Arc<RwLock<MergedTimeline>>, host: Host, m: WsMsg| async move {
            match m {
                WsMsg::Channel(WsMsgChannelBody::Note { id: _, body }) => {
                    let mut tl = tl.write().await;
                    tl.insert(host, HashSet::new(), body)
                        .await
                        .expect("TODO: handle error");
                }
            }
        };
        cxn.subscribe(move |m| f(tl.clone(), host1.clone(), m))
            .await;
        cxn.spawn().await.expect("TODO: handle error");

        self.server_cxn_store
            .insert(host.clone(), Arc::new(RwLock::new(cxn)));

        match fetch_hybrid_notes(host, api_key).await {
            Ok(notes) => {
                let mut tl = self.merged_timeline.write().await;
                for note in notes.into_iter().rev() {
                    tl.insert(host.clone(), HashSet::new(), note)
                        .await
                        .expect("TODO: handle error");
                }
            }
            Err(e) => {
                tracing::error!("failed to fetch notes: {e}");
            }
        }
    }
}

async fn fetch_hybrid_notes(host: &Host, api_key: &str) -> Result<Vec<Note>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://{}/api/notes/hybrid-timeline",
            host.to_string()
        ))
        .json(&json!({ "i": api_key }))
        .send()
        .await?
        .error_for_status()?;

    let res = res.json().await?;
    Ok(res)
}
