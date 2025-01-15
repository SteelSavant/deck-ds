use crate::{
    macros::newtype_uuid,
    pipeline::action::session_handler::{Pos, Size},
};
use std::{
    fmt::Debug,
    sync::{mpsc::Sender, Arc, Mutex},
    time::Duration,
};

use super::*;
use dbus::{
    blocking::{Connection, Proxy, SyncConnection},
    channel::MatchingReceiver,
    message::MatchRule,
};
use serde::Deserialize;
use xrandr::indexmap::IndexMap;

use std::io::Write;

pub type KWinScreenTrackingUpdateHandler = Box<dyn Fn(&[KWinScreenInfo]) + Send + Sync>;
newtype_uuid!(KWinScreenTrackingUpdateHandle);
pub struct KWinScreenTrackingScope {
    script_name: uuid::Uuid,
    script_id: i32,
    kwin_conn: Connection,
    kill_tx: Sender<()>,
    update_handles:
        Arc<Mutex<IndexMap<KWinScreenTrackingUpdateHandle, KWinScreenTrackingUpdateHandler>>>,
}

impl KWinScreenTrackingScope {
    pub fn new() -> Result<Self> {
        let script_name = uuid::Uuid::new_v4();
        let kwin_conn = Connection::new_session()?;

        let kwin_proxy = Self::get_kwin_proxy(&kwin_conn);

        let self_conn = SyncConnection::new_session()?;

        let script_text = Self::get_script_text(&self_conn.unique_name().to_string(), script_name);

        let mut script_file = tempfile::NamedTempFile::with_prefix("DeckDS-screentracking-")?;

        script_file.write_all(script_text.as_bytes())?;

        let script_file_path = script_file.into_temp_path();

        let script_id: i32;
        (script_id,) = kwin_proxy.method_call(
            "org.kde.kwin.Scripting",
            "loadScript",
            (script_file_path.to_str().unwrap(), &script_name.to_string()),
        )?;

        log::debug!("started screen tracking scipt id: {script_id} @ {script_file_path:?}");

        let (kill_tx, kill_rx) = std::sync::mpsc::channel::<()>();

        let res = Self {
            script_id,
            script_name,
            kwin_conn,
            kill_tx,
            update_handles: Arc::new(Default::default()),
        };

        let update_handles = res.update_handles.clone();

        // setup message receiver
        std::thread::spawn(move || {
            let receiver = self_conn.start_receive(
                MatchRule::new_method_call(),
                Box::new(move |message, _connection| -> bool {
                    log::trace!("got screen state dbus message: {:?}", message);

                    if let Some(_member) = message.member() {
                        let (token, arg) = message.get2::<String, String>();

                        let token =
                            uuid::Uuid::parse_str(&token.expect("expected token from dbus"))
                                .expect("dbus token should be valid uuid");

                        if token != script_name {
                            log::trace!("tokens don't match; {token} != {script_name} ; returning");
                            return false;
                        }

                        let arg = arg.expect("dbus arg should be valid");

                        let info = serde_json::from_str::<Vec<KWinScreenInfo>>(&arg)
                            .expect("json from dbus should parse");

                        log::trace!("updated screens for {} to {:?}", script_name, info);
                        let lock = update_handles
                            .lock()
                            .expect("Update Handles should not be poisoned");
                        for handle in lock.values() {
                            handle(&info);
                        }
                    } else {
                        log::warn!("no dbus member");
                    }
                    true
                }),
            );

            let mut signal = kill_rx.try_recv();

            while matches!(signal, Err(std::sync::mpsc::TryRecvError::Empty)) {
                self_conn.process(Duration::from_millis(1000)).unwrap();
                signal = kill_rx.try_recv();
            }

            log::trace!("Got screen state end signal value: {}", signal.is_ok());

            self_conn.stop_receive(receiver);
        });

        let script_proxy = res.get_script_proxy();

        script_proxy.method_call::<(), _, _, _>("org.kde.kwin.Script", "run", ())?;

        Ok(res)
    }

    pub fn register_update(
        &mut self,
        f: KWinScreenTrackingUpdateHandler,
    ) -> KWinScreenTrackingUpdateHandle {
        let handle = KWinScreenTrackingUpdateHandle::new();

        let mut lock = self
            .update_handles
            .lock()
            .expect("Update Handles should not be poisoned");

        lock.insert(handle, f);
        handle
    }

    pub fn unregister_update(&mut self, handle: KWinScreenTrackingUpdateHandle) {
        let mut lock = self
            .update_handles
            .lock()
            .expect("Update Handles should not be poisoned");

        lock.remove(&handle);
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
console.log("!!!!!! Matching screens for {dbus_addr} !!!!!!");

function parseScreensSection(info) {{
    const rxp = /Screens\n\s*=+\n((\n|.)*)\s*[A-Z][a-z]+\n\s*=+/gm;
    const m = rxp.exec(info);
    if (m) {{
        return m[1];
    }}
    return null;
}}

function parseScreens(screensSection) {{
    const rxp = /Screen\s+(\d+):\n\s*-+\n((?:\s*[A-Za-z\s]+:\s*.+\n)+)/gm;
    let match;
    const screens = [];  

    while ((match = rxp.exec(screensSection)) !== null) {{
        const screenNumber = match[1];
        const screenDetails = match[2];

        const screenFields = {{}};

        const fields = screenDetails.trim().split('\n');
        fields.forEach(field => {{
            const [key, value] = field.split(':').map(s => s.trim());
            let parsed = value;
            switch(key) {{
                case 'Enabled':
                    parsed = value === '1';
                    break;
                case 'Geometry':
                    geometry_rxp = /(\d+),(\d+),(\d+)x(\d+)/gm
                    m = geometry_rxp.exec(value);
                    screenFields['pos'] = {{
                            x: parseInt(m[1]),
                            y: parseInt(m[2]),
                    }};
                    screenFields['size'] = {{
                            w: parseInt(m[3]),
                            h: parseInt(m[4]),
                    }};
                    // exit early since we're using different keys
                    return;
            }}
            screenFields[key.toLowerCase()] = parsed;
        }});

        screenFields['id'] = screenNumber;

        // Assign this screen's field object to the screens object with the screen number as the key
        screens.push(screenFields);
    }}

    return screens;
}}

function getScreenInfo() {{
    const info = workspace.supportInformation();
    const screensSection = parseScreensSection(info);
    const screens = parseScreens(screensSection);

    return screens;
}}

function updateScreenInfo() {{
    const screenInfo = getScreenInfo();
    const stringified = JSON.stringify(screenInfo);
    console.log('updating info for', screenInfo.length, 'screens');
    console.log('sending msg over dbus:', stringified);

    callDBus("{dbus_addr}", "/", "", "updateScreens", "{script_name}", stringified);
}}

workspace.numberScreensChanged.connect(updateScreenInfo);
workspace.virtualScreenGeometryChanged.connect(updateScreenInfo);
workspace.virtualScreenSizeChanged.connect(updateScreenInfo);

updateScreenInfo();
"#
        )
    }
}

impl Drop for KWinScreenTrackingScope {
    fn drop(&mut self) {
        let _: Result<(), _> = Self::get_kwin_proxy(&self.kwin_conn).method_call(
            "org.kde.kwin.Scripting",
            "unloadScript",
            (&self.script_name.to_string(),),
        );

        let _ = self.kill_tx.send(());
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KWinScreenInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub pos: Pos,
    pub size: Size,
}

pub struct KWinScreenStateMatcher {
    pub name: String,
    pub enabled: Option<bool>,
    pub pos: Option<Pos>,
    pub size: Option<Size>,
}
