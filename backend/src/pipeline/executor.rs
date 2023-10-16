use std::{path::PathBuf, collections::HashMap};


use super::{common::{Context, Dependency}, config::{PipelineDefinition, SelectionType, PipelineAction}, action::virtual_screen::VirtualScreen};

pub struct PipelineExecutor {
    ctx: Context,
}

impl PipelineExecutor {
    pub fn new(defaults_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            ctx: Context {
                defaults_dir,
                config_dir,
                executors: HashMap::from([("VirtualScreen".to_string(), Box::new( VirtualScreen))])
            },
        }
    }

    pub fn exec(&self, pipeline: &PipelineDefinition) {
        let actions = pipeline.actions.iter().filter(|a| matches!(a.optional, Some(true) | None)).collect::<Vec<_>>();
        

        
        for action in actions {
            action.value.deps(&self.ctx)?;
            action.value.startup()?;
        }

        println!("app exec!");



        for action in pipeline.actions.iter() {
            action.value.teardown()?;
        }

        let res = Dependency::TrueVideoWall.install(&self.ctx);
        println!("{:?}", res)
    }
}

enum ActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl SelectionType<PipelineAction> {
    fn exec(&self, action: ActionType) {
        match self {
            SelectionType::Single(value) => value.exec(action),
            SelectionType::OneOf(values, key) =>  match values.get(key) {
                Some(v) => v.exec(action),
                None => todo!(),
            },
            SelectionType::AnyOf(values, keys) => {
                values.g
            },
        }
    }
}

impl PipelineAction {
    fn exec(&self, action: ActionType) {
        match action {
            ActionType::Dependencies => self.,
            ActionType::Setup => todo!(),
            ActionType::Teardown => todo!(),
        }
    }
}
