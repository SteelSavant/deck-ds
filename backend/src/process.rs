use anyhow::{Context, Result};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use regex::Regex;
use std::{
    io::BufRead,
    process::Command,
    time::{Duration, Instant},
};

pub struct AppProcess {
    pid: Pid,
}

impl AppProcess {
    // TODO::search by app args; for now, assume the first "SteamLaunch is what we want"
    pub fn find(timeout: Duration) -> Result<Self> {
        let pid = {
            let now = Instant::now();
            let mut pid = Self::find_app_process();
            while now.elapsed() < timeout && pid.is_err() {
                pid = Self::find_app_process();
                std::thread::sleep(Duration::from_millis(100));
            }
            pid
        }?;

        Ok(Self { pid })
    }

    pub fn is_alive(&self) -> bool {
        Self::is_pid_alive(self.pid)
    }

    fn is_pid_alive(pid: Pid) -> bool {
        signal::kill(pid, None).is_ok()
    }

    /// Kills the application process (very aggressively)
    pub fn kill(self) -> Result<()> {
        fn kill_timeout(pid: Pid, signal: Signal, timeout: Duration, kill_group: bool) -> bool {
            let join_handle = std::thread::spawn(move || {
                if kill_group {
                    signal::killpg(pid, signal)
                } else {
                    signal::kill(pid, signal)
                }
            });
            std::thread::sleep(timeout);
            if join_handle.is_finished(){
                let res = join_handle.join();
                matches!(res, Ok(Ok(()))) && !AppProcess::is_pid_alive(pid)
            } else {
                false
            }
        }

        let pstree = Self::get_process_tree(Some(self.pid))?;
        let mut ids: Vec<_> = pstree
            .iter()
            .skip(1)
            .filter(|f| !f.contains("bwrap") && !f.contains("xdg-dbus"))
            .map(|branch| {
                (
                    Self::get_pid_from_branch(branch).expect("branch should have pid"),
                    branch,
                )
            })
            .collect();

        let mut stack = vec![];

        for signal in [Signal::SIGTERM, Signal::SIGHUP, Signal::SIGKILL] {
            if !ids.is_empty() {
                println!("sending signal {signal} to process tree");
            }

            for (pid, branch) in ids.into_iter() {
                if Self::is_pid_alive(pid) {
                    println!("sending signal {signal} to process {pid}");
                    let timeout = Duration::from_millis(100);
                    if !kill_timeout(pid, signal, timeout, false) {
                        stack.push((pid, branch))
                    }
                }
            }

            let timeout = Duration::from_secs(2);
            let now = Instant::now();

            ids = stack;
            stack = vec![];

            while now.elapsed() < timeout && !ids.is_empty() {
                for (pid, branch) in ids {
                    if Self::is_pid_alive(pid) {
                        stack.push((pid, branch))
                    }
                }

                ids = stack;
                stack = vec![];

                std::thread::sleep(timeout / (ids.len() + 1).try_into().unwrap_or(1));
            }
        }

        if ids.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Processes failed to exit in time: {:?}",
                ids.into_iter()
                    .map(|(_, branch)| branch)
                    .collect::<Vec<_>>(),
            ))
        }
    }

    fn get_pid_from_branch(branch: &str) -> Result<Pid> {
        let rgx = Regex::new(r".*,(\d+)").expect("pid regex should compile");
        let pid = str::parse(
            rgx.captures(branch)
                .ok_or(anyhow::anyhow!("no pid captures found for {branch}\n"))?
                .get(1)
                .expect("pid regex should have capture")
                .as_str(),
        )?;

        Ok(Pid::from_raw(pid))
    }

    /// Gets the process tree as an interator of strings. If [pid] is Some, only gets process tree of that pid.
    fn get_process_tree(pid: Option<Pid>) -> Result<Vec<String>> {
        // let path = match pid {
        //     Some(id) => format!("./log.{id}.txt"),
        //     None => "./log.full.txt".to_string(),
        // };
        let pid = pid.unwrap_or(Pid::from_raw(1));
        let args = ["-apl", &pid.as_raw().to_string()];
        let output = Command::new("pstree")
            .args(args)
            .output()
            .with_context(|| format!("Could not fetch process tree for {pid}"))?;
        // std::fs::write(path, &output.stdout);

        Ok(output.stdout.lines().collect::<Result<Vec<_>, _>>()?)
    }

    fn find_app_process() -> Result<Pid> {
        let pstree = Self::get_process_tree(None)?;

        let search = "SteamLaunch AppId=";

        let mut branchs = pstree.iter().skip_while(|branch| !branch.contains(search));
        let child = branchs
            .next()
            .ok_or(anyhow::anyhow!("app process not found"))?;
        Self::get_pid_from_branch(child)
    }
}
