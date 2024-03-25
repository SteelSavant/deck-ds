use anyhow::Result;

use crate::sys::kwin::KWin;

use super::{ActionId, ActionImpl, ActionType};

pub mod primary_windowing;
pub mod secondary_app;

const SCRIPT: &str = "emulatorwindowing";

// This technically could be generic over the app type and just support one app,
// but it adds a lot of boilerplate elsewhere, and I don't feel like dealing with it.

trait OptionsRW {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized;
    fn write(&self, kwin: &KWin) -> Result<()>;
}
