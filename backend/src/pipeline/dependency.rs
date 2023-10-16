use super::common::Context;

pub mod true_video_wall;

use self::true_video_wall::TrueVideoWall;

#[enum_delegate::register]
pub trait DependencyExecutor {
    fn install(&self, ctx: &mut Context) -> Result<(), String>;
}

#[derive(Debug, Clone)]
#[enum_delegate::implement(DependencyExecutor)]
pub enum Dependency {
    TrueVideoWall(TrueVideoWall),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyId(pub String);
