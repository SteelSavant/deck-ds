use std::{
    process::Command,
    thread::sleep,
    time::{Duration, Instant},
};

use nix::unistd::Pid;

use anyhow::{Context, Result};
use regex::Regex;

pub struct WindowInfo {
    pub name: String,
    pub classes: Vec<String>,
}

pub fn get_active_window_info() -> Result<WindowInfo> {
    let cmd = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname", "getwindowclassname"])
        .output()?;

    if cmd.status.success() {
        let out = String::from_utf8_lossy(&cmd.stdout);
        let mut lines = out.lines();
        let name = lines.next()?.to_string();

        Ok(WindowInfo {
            name,
            classes: lines.map(|v| v.to_string()).collect(),
        })
    } else {
        let err = String::from_utf8_lossy(&cmd.stderr).to_string();
        Err(anyhow::anyhow!(err.clone()))
            .with_context(|| format!("failed to get info for active window: {err}"))
    }
}

pub fn get_window_info_from_pid(pid: Pid) -> Result<WindowInfo> {
    let id = get_window_id_from_pid(pid)?;
    get_window_info_for_window_id(id)
}

pub fn get_window_info_from_pid_default_active_after(
    pid: Pid,
    timeout: Duration,
) -> Result<WindowInfo> {
    std::thread::sleep(Duration::from_millis(500)); // Delay in case window caption changes rapidly on launch

    let mut window_info = get_window_info_from_pid(pid);

    let timer = Instant::now();

    while timer.elapsed() < timeout && window_info.is_err() {
        window_info = get_window_info_from_pid(pid);
        sleep(Duration::from_millis(100));
    }

    window_info.or_else(|_| get_active_window_info())
}

fn get_window_id_from_pid(pid: Pid) -> Result<usize> {
    let cmd = Command::new("xdotool")
        .args(["search", "--pid", &pid.as_raw().to_string()])
        .output()?;

    if cmd.status.success() {
        let out = String::from_utf8_lossy(&cmd.stdout);
        Ok(str::parse(&out).with_context(|| format!("failed to parse pid {}", out))?)
    } else {
        let err = String::from_utf8_lossy(&cmd.stderr).to_string();
        Err(anyhow::anyhow!(err.clone()))
            .with_context(|| format!("failed to get window id from pid: {err}"))
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
