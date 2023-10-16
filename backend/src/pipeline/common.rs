use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;

pub struct Context {
    /// path to directory containing contents of decky "defaults" folder
    pub defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
}
