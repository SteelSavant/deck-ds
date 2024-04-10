use anyhow::Result;

use crate::sys::kwin::KWin;

use super::{ActionId, ActionImpl, ActionType};

pub mod main_app_automatic_windowing;
pub mod primary_windowing;
pub mod secondary_app;

const SCRIPT: &str = "emulatorwindowing";

trait OptionsRW {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized;
    fn write(&self, kwin: &KWin) -> Result<()>;
}
