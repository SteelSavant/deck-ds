use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::path::Path;

use crate::decky_env::DeckyEnv;

fn version_filepath(decky_env: &DeckyEnv) -> std::path::PathBuf {
    decky_env.decky_plugin_settings_dir.join(".version")
}

pub fn save_version_file(decky_env: &DeckyEnv) -> std::io::Result<usize> {
    let path = version_filepath(decky_env);
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    std::fs::File::create(path)?.write(crate::consts::PACKAGE_VERSION.as_bytes())
}

pub fn read_version_file(decky_env: &DeckyEnv) -> String {
    let path = version_filepath(decky_env);
    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut read_version = String::new();
            match file.read_to_string(&mut read_version) {
                Ok(_) => read_version,
                Err(e) => {
                    log::warn!("Cannot read version file str: {}", e);
                    crate::consts::PACKAGE_VERSION.to_owned()
                }
            }
        }
        Err(e) => {
            log::warn!("Cannot read version file: {}", e);
            crate::consts::PACKAGE_VERSION.to_owned()
        }
    }
}

pub fn get_maybe_window_names_classes_from_title(title: &str) -> Vec<String> {
    use unicode_segmentation::UnicodeSegmentation;

    let title = title.to_string();
    let initials = title
        .split_whitespace()
        .filter_map(|v| v.graphemes(true).next())
        .collect::<Vec<_>>()
        .join("");

    vec![title, initials]
}

pub fn create_dir_all<A: AsRef<Path> + std::fmt::Debug>(path: A) -> Result<()> {
    if !path.as_ref().is_dir() {
        log::debug!("creating path {path:?}");
        std::fs::create_dir_all(&path)
            .with_context(|| format!("failed to create dirs for path {:?}", path))
    } else {
        Ok(())
    }
}

pub fn escape_string_for_regex(mut s: String) -> String {
    for c in [
        '\\', '^', '$', '*', '+', '?', '.', '(', ')', '|', '{', '}', '[', ']',
    ] {
        s = s.replace(c, &format!("\\{c}"));
    }

    s
}
