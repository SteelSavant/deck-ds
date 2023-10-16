use std::collections::HashMap;
use std::path::PathBuf;

use super::dependency::{Dependency, DependencyId};

pub struct Context {
    /// path to directory containing contents of decky "defaults" folder
    pub defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
    /// known dependencies
    pub dependencies: HashMap<DependencyId, Dependency>,
}
