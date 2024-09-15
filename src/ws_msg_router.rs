use std::collections::{HashMap, HashSet};

use crate::common_types::BranchKey;

#[derive(Debug, Default)]
pub struct WsMsgRouter {
    channel_id_to_branches: HashMap<String, HashSet<BranchKey>>,
}

impl WsMsgRouter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn extend(&mut self, channel_id: String, branches: impl IntoIterator<Item = BranchKey>) {
        self.channel_id_to_branches
            .entry(channel_id)
            .or_default()
            .extend(branches.into_iter());
    }

    pub fn solve_branches(&self, channel_id: &str) -> HashSet<BranchKey> {
        self.channel_id_to_branches
            .get(channel_id)
            .cloned()
            .unwrap_or_default()
    }
}
