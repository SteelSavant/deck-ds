pub mod emulator_windowing;
pub mod true_video_wall;

use self::emulator_windowing::EmulatorWindowing;
use self::true_video_wall::TrueVideoWall;

use anyhow::Result;

use super::executor::PipelineContext;

#[enum_delegate::register]
pub trait DependencyExecutor {
    fn install(&self, ctx: &mut PipelineContext) -> Result<()>;
}

#[derive(Debug, Clone)]
#[enum_delegate::implement(DependencyExecutor)]
pub enum Dependency {
    TrueVideoWall(TrueVideoWall),
    EmulatorWindowing(EmulatorWindowing),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyId(pub String);
