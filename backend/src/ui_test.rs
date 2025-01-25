use anyhow::Result;
use std::{sync::Arc, time::Duration};

use crate::{
    config::GlobalConfig,
    decky_env::DeckyEnv,
    pipeline::{
        action::{
            multi_window::primary_windowing::{GeneralOptions, MultiWindow},
            session_handler::DesktopSessionHandler,
            ActionId, ActionImpl,
        },
        executor::PipelineContext,
    },
    settings_db::SettingsDb,
};

#[allow(dead_code)]
pub fn ui_test(decky_env: Arc<DeckyEnv>) -> Result<()> {
    let ctx = &mut PipelineContext::new(
        None,
        GlobalConfig::default(),
        SettingsDb::new("memory"),
        decky_env,
    );

    let ui = DesktopSessionHandler;

    let vscreen = MultiWindow {
        id: ActionId::nil(),
        general: GeneralOptions::default(),
        cemu: None,
        citra: None,
        dolphin: None,
        custom: None,
    };

    let duration = Duration::from_secs(10);
    println!("setting up ui...");
    ui.setup(ctx)?;
    println!("waiting");
    std::thread::sleep(duration);
    println!("setting up vscreen");
    vscreen.setup(ctx)?;
    println!("waiting");
    std::thread::sleep(duration);
    println!("tearing down vscreen");
    vscreen.teardown(ctx)?;
    println!("waiting");
    std::thread::sleep(duration);
    println!("tearing down ui");
    ui.teardown(ctx)?;
    println!("waiting");
    std::thread::sleep(duration);
    println!("done");
    Ok(())
}
