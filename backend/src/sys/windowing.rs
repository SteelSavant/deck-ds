use std::process::Command;

use nix::unistd::Pid;

use anyhow::{Context, Result};
use regex::Regex;

pub struct WindowInfo {
    pub name: String,
    pub classes: Vec<String>,
}

pub fn get_window_info_from_pid(pid: Pid) -> Result<WindowInfo> {
    let id = get_window_id_from_pid(pid)?;
    get_window_info_for_window_id(id)
}

fn get_window_id_from_pid(pid: Pid) -> Result<usize> {
    let cmd = Command::new("xdotool")
        .args(["search", "--pid", &pid.as_raw().to_string()])
        .output()?;

    if cmd.status.success() {
        Ok(str::parse(&String::from_utf8_lossy(&cmd.stdout))?)
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&cmd.stderr).to_string()
        ))
    }
}

fn get_window_info_for_window_id(id: usize) -> Result<WindowInfo> {
    let cmd = Command::new("xprop")
        .args(["-id", &id.to_string()])
        .output()?;

    // TODO::make these static/const
    let name_regex = Regex::new(r#"^WM_NAME\(STRING\) = "(.+)"$"#).unwrap();
    let classes_regex = Regex::new(r#"^WM_CLASS\(STRING\) = (.+)$"#).unwrap();

    if cmd.status.success() {
        let out = String::from_utf8_lossy(&cmd.stdout);
        let name = name_regex
            .captures(&out)
            .with_context(|| "failed to find captures for window name")?
            .get(1)
            .expect("window name regex should have 1 capture")
            .as_str()
            .to_string();

        let classes = classes_regex
            .captures(&out)
            .with_context(|| "failed to find captures for window classes")?
            .get(1)
            .expect("window class regex should have 1 capture")
            .as_str()
            .split_terminator(',')
            .map(|v| v.trim_matches(&[' ', '"'] as &[_]).to_string())
            .collect();

        Ok(WindowInfo { name, classes })
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&cmd.stderr).to_string()
        ))
    }
}
