use crate::pipeline::common::PipelineActionExecutor;

use super::super::common::Dependency;
struct VirtualScreen;

impl PipelineActionExecutor for VirtualScreen {
    fn get_dependencies() -> Vec<Dependency> {
        vec![Dependency::TrueVideoWall]
    }
}
