use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use anyhow::Result;
use egui::ahash::HashSet;
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DefaultOnError;

use crate::decky_env::DeckyEnv;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ClientTeardownAction {
    MainAppAutomaticWindowing {
        app_id: u32,
        action_id: String,
        previous_launch_options: String,
    },
}

impl ClientTeardownAction {
    fn get_id(&self) -> &String {
        match self {
            ClientTeardownAction::MainAppAutomaticWindowing { action_id: id, .. } => id,
        }
    }
}

pub struct ClientPipelineHandler {
    /// Decky environment variables for the session
    decky_env: Arc<DeckyEnv>,
    state: ClientPipelineState,
}

#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]

struct ClientPipelineState {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    teardown: IndexMap<String, ClientTeardownAction>,
}

impl ClientPipelineHandler {
    pub fn new(decky_env: Arc<DeckyEnv>) -> Self {
        let path = get_state_path(&decky_env);
        let state = std::fs::read_to_string(path)
            .context("failed to read previous client state")
            .and_then(|v| serde_json::from_str(&v).context("failed to parse previous client state"))
            .unwrap_or_default();

        Self { decky_env, state }
    }
    pub fn add_client_teardown_action(&mut self, action: ClientTeardownAction) -> Result<()> {
        self.state.teardown.insert(action.get_id().clone(), action);

        self.save_state()
    }

    pub fn remove_client_teardown_actions(&mut self, ids: HashSet<String>) -> Result<()> {
        self.state.teardown.retain(|_, v| !ids.contains(v.get_id()));

        self.save_state()
    }

    pub fn get_client_teardown_actions(&self) -> Vec<ClientTeardownAction> {
        return self.state.teardown.values().map(|v| v.clone()).collect();
    }

    fn save_state(&self) -> Result<()> {
        let path = get_state_path(&self.decky_env);
        std::fs::write(path, serde_json::to_string_pretty(&self.state)?)?;

        Ok(())
    }
}

fn get_state_path(decky_env: &DeckyEnv) -> PathBuf {
    decky_env.decky_plugin_runtime_dir.join("client_state.json")
}
