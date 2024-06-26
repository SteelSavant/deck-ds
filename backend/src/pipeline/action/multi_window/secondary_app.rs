mod launch_secondary_app_preset;
mod launch_secondary_flatpak_app;
mod secondary_app_options;

pub use launch_secondary_app_preset::*;
pub use launch_secondary_flatpak_app::*;
use nix::unistd::Pid;
use schemars::JsonSchema;
pub use secondary_app_options::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub enum SecondaryAppScreenPreference {
    #[default]
    PreferSecondary,
    PreferPrimary,
}

#[derive(Debug, Default, Clone, Copy, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub enum SecondaryAppWindowingBehavior {
    #[default]
    Fullscreen,
    Maximized,
    Minimized,
    Unmanaged,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecondaryAppState {
    pid: Option<Pid>,
    options: SecondaryAppWindowOptions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SerializableSecondaryAppState {
    options: SecondaryAppWindowOptions,
}

impl Serialize for SecondaryAppState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableSecondaryAppState {
            options: self.options.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SecondaryAppState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerializableSecondaryAppState::deserialize(deserializer).map(|v| SecondaryAppState {
            pid: None,
            options: v.options,
        })
    }
}
