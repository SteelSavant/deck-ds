use anyhow::Result;
use std::{sync::Arc, time::Duration};

use crate::{
    decky_env::DeckyEnv,
    pipeline::{
        self,
        action::{
            multi_window::primary_windowing::{GeneralOptions, MultiWindow},
            session_handler::{DesktopSessionHandler, ExternalDisplaySettings},
            ActionId, ActionImpl,
        },
        executor::PipelineContext,
    },
    sys,
};

#[allow(dead_code)]
pub fn ui_test(decky_env: Arc<DeckyEnv>) -> Result<()> {
    use sys::x_display::{ModePreference, Resolution};

    let ctx = &mut PipelineContext::new(decky_env);

    let ui = DesktopSessionHandler {
        id: ActionId::nil(),
        teardown_external_settings: ExternalDisplaySettings::Preference(ModePreference {
            resolution: sys::x_display::ModeOption::Exact(Resolution { w: 1920, h: 1080 }),
            aspect_ratio: sys::x_display::AspectRatioOption::Any,
            refresh: sys::x_display::ModeOption::AtLeast(60.),
        }),
        teardown_deck_location: Some(pipeline::action::session_handler::RelativeLocation::Below),
        deck_is_primary_display: true,
    };

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
