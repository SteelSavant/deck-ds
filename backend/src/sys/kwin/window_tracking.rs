// Window tracking adapted from https://github.com/jinliu/kdotool
use std::{
    cmp::Ordering,
    ops::Deref,
    sync::{mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use super::*;
use dbus::{
    blocking::{Connection, Proxy, SyncConnection},
    channel::MatchingReceiver,
    message::MatchRule,
};
use serde::Deserialize;
use str_distance::str_distance_normalized;

use std::io::Write;

pub struct KWinNewWindowTrackingScope {
    script_name: uuid::Uuid,
    script_id: i32,
    kwin_conn: Connection,
    msg_thread: Option<JoinHandle<Option<KWinClientInfo>>>,
    kill_tx: Sender<Option<KWinClientMatcher>>,
}

impl KWinNewWindowTrackingScope {
    pub fn new() -> Result<Self> {
        let script_name = uuid::Uuid::new_v4();
        let kwin_conn = Connection::new_session()?;

        let kwin_proxy = Self::get_kwin_proxy(&kwin_conn);

        let self_conn = SyncConnection::new_session()?;

        let script_text = Self::get_script_text(&self_conn.unique_name().to_string(), script_name);

        let mut script_file = tempfile::NamedTempFile::with_prefix("DeckDS-windowtracking-")?;

        script_file.write_all(script_text.as_bytes())?;

        let script_file_path = script_file.into_temp_path();

        let script_id: i32;
        (script_id,) = kwin_proxy.method_call(
            "org.kde.kwin.Scripting",
            "loadScript",
            (script_file_path.to_str().unwrap(), &script_name.to_string()),
        )?;

        log::debug!("started window tracking scipt id: {script_id} @ {script_file_path:?}");

        let (kill_tx, kill_rx) = std::sync::mpsc::channel::<Option<KWinClientMatcher>>();
        // setup message receiver
        let msg_thread = std::thread::spawn(move || {
            fn get_best_match<'a>(
                maybe_strings: &[String],
                clients: &'a [KWinClientInfo],
            ) -> Option<&'a KWinClientInfo> {
                struct Match<'a> {
                    caption_score: f64,
                    window_class_score: f64,
                    client: &'a KWinClientInfo,
                }

                let matches = clients.iter().filter_map(|c| {
                    const THRESH: f64 = 0.4;

                    let caption_score = maybe_strings.iter().fold(1., |acc, item| {
                        str_distance_normalized(
                            item.to_lowercase(),
                            c.caption.to_lowercase(),
                            str_distance::JaroWinkler::default(),
                        )
                        .min(acc)
                    });

                    let window_class_score = maybe_strings.iter().fold(1., |acc, item| {
                        c.window_classes
                            .iter()
                            .fold(1., |acc, wc| {
                                str_distance_normalized(
                                    item.to_lowercase(),
                                    wc.to_lowercase(),
                                    str_distance::JaroWinkler::default(),
                                )
                                .min(acc)
                            })
                            .min(acc)
                    });

                    log::debug!(
                        "window client {c:?} has scores ({caption_score},{window_class_score})"
                    );

                    if caption_score < THRESH || window_class_score < THRESH {
                        Some(Match {
                            caption_score,
                            window_class_score,
                            client: c,
                        })
                    } else {
                        None
                    }
                });

                matches
                    .reduce(|acc, m| {
                        let acc_score = acc.caption_score + acc.window_class_score;
                        let m_score = m.caption_score + m.window_class_score;

                        if acc_score < m_score {
                            acc
                        } else {
                            m
                        }
                    })
                    .map(|v| v.client)
            }

            let info: Arc<Mutex<Vec<KWinClientInfo>>> = Arc::new(Mutex::new(vec![]));

            let info_ref = info.clone();
            let receiver =
                self_conn.start_receive(
                    MatchRule::new_method_call(),
                    Box::new(move |message, _connection| -> bool {
                        log::trace!("got dbus message: {:?}", message);

                        if let Some(_member) = message.member() {
                            let (token, arg) = message.get2::<String, String>();

                            let token =
                                uuid::Uuid::parse_str(&token.expect("expected token from dbus"))
                                    .expect("dbus token should be valid uuid");

                            if token != script_name {
                                log::trace!(
                                    "tokens don't match; {token} != {script_name} ; returning"
                                );
                                return false;
                            }

                            let arg = arg.expect("dbus arg should be valid");

                            if let Ok(mut lock) = info_ref.try_lock() {
                                let mut info = serde_json::from_str::<Vec<KWinClientInfo>>(&arg)
                                    .expect("json from dbus should parse");

                                std::mem::swap(lock.as_mut(), &mut info);
                                log::trace!(
                                    "updated client windows for {} to {:?}",
                                    script_name,
                                    lock,
                                );
                            }
                        }
                        true
                    }),
                );

            let mut signal = kill_rx.try_recv();

            while matches!(signal, Err(std::sync::mpsc::TryRecvError::Empty)) {
                self_conn.process(Duration::from_millis(1000)).unwrap();
                signal = kill_rx.try_recv();
            }

            log::trace!("Got end signal value: {}", signal.is_ok());

            if let Ok(Some(matcher)) = signal {
                let mut found_instant: Option<Instant> = None;
                let timeout_instant = Instant::now();

                while timeout_instant.elapsed() < matcher.max_delay
                    && (found_instant.is_none()
                        || found_instant.unwrap().elapsed() < matcher.min_delay)
                {
                    let has_match = {
                        let lock = info.lock().unwrap();
                        get_best_match(&matcher.maybe_strings, lock.deref()).is_some()
                    };

                    if has_match {
                        if found_instant.is_none() {
                            found_instant = Some(Instant::now());
                        }
                    } else if found_instant.is_some() {
                        found_instant = None;
                    }

                    self_conn.process(Duration::from_millis(100)).unwrap();
                }

                self_conn.stop_receive(receiver);

                let lock = info.lock().unwrap();
                let best = get_best_match(&matcher.maybe_strings, lock.deref());

                best.or_else(|| match matcher.preferred_ord_if_no_match {
                    Ordering::Less => lock.deref().iter().next(),
                    Ordering::Equal => None,
                    Ordering::Greater => lock.deref().iter().last(),
                })
                .cloned()
            } else {
                self_conn.stop_receive(receiver);
                None
            }
        });

        let res = Self {
            script_id,
            script_name,
            kwin_conn,
            msg_thread: Some(msg_thread),
            kill_tx,
        };

        let script_proxy = res.get_script_proxy();

        script_proxy.method_call("org.kde.kwin.Script", "run", ())?;

        Ok(res)
    }

    pub fn get_best_window_client(
        mut self,
        matcher: KWinClientMatcher,
    ) -> Result<Option<KWinClientInfo>> {
        log::trace!("joining windowing thread");

        self.kill_tx.send(Some(matcher))?;

        let window = self
            .msg_thread
            .take()
            .unwrap()
            .join()
            .map_err(|err| anyhow::anyhow!("failed to join dbus message thread: {err:#?}"))?;

        log::trace!("using client window: {window:?}");

        Ok(window)
    }

    fn get_kwin_proxy(kwin_conn: &Connection) -> Proxy<&Connection> {
        kwin_conn.with_proxy("org.kde.KWin", "/Scripting", Duration::from_secs(10))
    }

    fn get_script_proxy(&self) -> Proxy<&Connection> {
        let is_kde5 = matches!(std::env::var("KDE_SESSION_VERSION").as_deref(), Ok("5"));

        self.kwin_conn.with_proxy(
            "org.kde.KWin",
            if is_kde5 {
                format!("/{}", self.script_id)
            } else {
                format!("/Scripting/Script{}", self.script_id)
            },
            Duration::from_millis(5000),
        )
    }

    fn get_script_text(dbus_addr: &str, script_name: uuid::Uuid) -> String {
        format!(
            r#"
console.log("!!!!!! Matching windows for {dbus_addr} !!!!!!");

let clients = [];

function updateClients() {{
    try {{
        const badWindows = [{{
            caption: 'gamescope',
            windowClass: 'gamescope'
        }}, {{
            caption: 'Steam',
            windowClass: 'steam'
        }}, {{
            caption: 'Steam',
            windowClass: ''
        }}];

        const filteredClients = clients.filter((c) => !badWindows.find((bw) => {{
            try {{
                const windowClass = c.resourceClass.toString().toLowerCase();
                return bw.caption === c.caption && windowClass.includes(bw.windowClass);
            }} catch(ex) {{
                return false;
            }}
            
        }})).map((client) => {{
            return {{
                id: client.windowId,
                caption: client.caption,
                window_classes: client.resourceClass.toString().toLowerCase().split(' ')
            }}
        }});

        const stringified = JSON.stringify(filteredClients);
        console.log('sending msg over dbus:', stringified);

        callDBus("{dbus_addr}", "/", "", "updateClients", "{script_name}", stringified);
    }} catch(ex) {{
        console.error('update clients threw', ex);
    }}
}}

workspace.clientAdded.connect((client) => {{
    if (!client.normalWindow) return;

    client.captionChanged.connect(updateClients);

    console.log('matcher got new client', client.caption);

 
    clients = [...clients, client];
    updateClients();
}});

workspace.clientRemoved.connect((client) => {{
    console.log('matcher removed client');

    clients = clients.filter((c) => c.id !== client.windowId);
    updateClients();
}});
"#
        )
    }
}

impl Drop for KWinNewWindowTrackingScope {
    fn drop(&mut self) {
        let _: Result<(), _> = Self::get_kwin_proxy(&self.kwin_conn).method_call(
            "org.kde.kwin.Scripting",
            "unloadScript",
            (&self.script_name.to_string(),),
        );

        let _ = self.kill_tx.send(None);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KWinClientInfo {
    pub caption: String,
    pub window_classes: Vec<String>,
}
pub struct KWinClientMatcher {
    /// minimum delay to wait for new windows after finding a matcher
    pub min_delay: Duration,
    /// maximum delay to wait for windows
    pub max_delay: Duration,
    /// If no match found, where to pull from as a default; `Ordering::Equal` if no default, otherwise `Ordering::Less` for first found window, `Ordering::Greater` for last found window.
    pub preferred_ord_if_no_match: Ordering,
    /// Possible strings for either the window title or class. We're guessing here.
    pub maybe_strings: Vec<String>,
}
