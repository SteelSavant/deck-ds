use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::Command;
use typemap::{Key, TypeMap};

use super::dependency::{Dependency, DependencyId};

use super::{
    action::{PipelineAction, PipelineActionExecutor},
    config::{PipelineDefinition, SelectionType},
    dependency::{true_video_wall::TrueVideoWall, DependencyExecutor},
};

pub struct PipelineExecutor {
    ctx: PipelineContext,
}

pub struct PipelineContext {
    /// path to directory containing contents of decky "defaults" folder
    pub defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
    /// known dependencies
    pub dependencies: HashMap<DependencyId, Dependency>,
    /// pipeline state
    pub state: TypeMap,
}

impl PipelineExecutor {
    pub fn new(defaults_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            ctx: PipelineContext {
                defaults_dir,
                config_dir,
                dependencies: HashMap::from([(
                    TrueVideoWall::id(),
                    Dependency::TrueVideoWall(TrueVideoWall),
                )]),
                state: TypeMap::new(),
            },
        }
    }

    pub fn exec(&mut self, pipeline: &PipelineDefinition) -> Result<()> {
        let res = self.build(pipeline);

        match res {
            Err(err) => Err(anyhow::anyhow!("Encountered errors: {:?}", vec![err])),
            Ok(pipeline) => {
                let mut run = vec![];
                let mut errors = vec![];

                for action in pipeline {
                    let res = action
                        .exec(&mut self.ctx, ActionType::Dependencies)
                        .and_then(|_| {
                            run.push(action);
                            run.last()
                                .expect("action should exist")
                                .exec(&mut self.ctx, ActionType::Setup)
                        });

                    if let Err(err) = res {
                        errors.push(err);
                        break;
                    }
                }

                if errors.is_empty() {
                    // let appid = 12146987087370911744u64;

                    // if let Err(_) = Command::new("steam")
                    //     .args([format!("steam://rungameid/{appid}")])
                    //     .status()
                    // {
                    //     errors.push(anyhow!("failed to run game in steam"));
                    // }

                    println!("Running App!");
                }

                for action in run.into_iter().rev() {
                    let ctx = &mut self.ctx;

                    let res = action.exec(ctx, ActionType::Teardown);
                    if let Err(err) = res {
                        errors.push(err);
                    }
                }

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Encountered errors: {:?}", errors))
                }
            }
        }
    }

    fn build(&self, pipeline: &PipelineDefinition) -> Result<Vec<PipelineAction>> {
        pipeline
            .actions
            .iter()
            .map(|s| {
                if matches!(s.optional, Some(true) | None) {
                    match &s.value {
                        SelectionType::Single(a) => Ok(vec![a.clone()]),
                        SelectionType::OneOf(values, key) => values
                            .get(key)
                            .ok_or(anyhow!("missing action {key}"))
                            .map(|a| vec![a.clone()]),
                        SelectionType::AnyOf(values, keys) => {
                            let mut ordered = keys
                                .iter()
                                .map(|k| {
                                    values
                                        .get_index_of(k)
                                        .map(|i| (i, k))
                                        .ok_or_else(|| anyhow!("missing action {k}"))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            ordered.sort_by_key(|v| v.0);

                            Ok(ordered
                                .into_iter()
                                .map(|(_, k)| values[k].clone())
                                .collect())
                        }
                    }
                } else {
                    Ok(vec![])
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.into_iter().flatten().collect())
    }
}

enum ActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl PipelineAction {
    fn exec(&self, ctx: &mut PipelineContext, action: ActionType) -> Result<()> {
        match action {
            ActionType::Dependencies => {
                let ids = self.get_dependencies();

                let deps = ids
                    .iter()
                    .map(|id: &super::dependency::DependencyId| {
                        ctx.dependencies
                            .get(id)
                            .map(|d| (*d).clone())
                            .ok_or_else(|| anyhow!("missing dependency {id:?}"))
                    })
                    .collect::<Result<Vec<_>>>()?;

                for d in deps {
                    // TODO::consider tracking installs to avoid reinstalling dependencies
                    d.install(ctx)?;
                }

                Ok(())
            }
            ActionType::Setup => self.setup(ctx),
            ActionType::Teardown => self.tear_down(ctx),
        }
    }
}



