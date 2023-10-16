use std::{collections::HashMap, path::PathBuf};

use super::{
    action::{PipelineAction, PipelineActionExecutor},
    common::Context,
    config::{PipelineDefinition, SelectionType},
    dependency::{true_video_wall::TrueVideoWall, Dependency, DependencyExecutor},
};

pub struct PipelineExecutor {
    ctx: Context,
}

impl PipelineExecutor {
    pub fn new(defaults_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            ctx: Context {
                defaults_dir,
                config_dir,
                dependencies: HashMap::from([(
                    TrueVideoWall::id(),
                    Dependency::TrueVideoWall(TrueVideoWall),
                )]),
            },
        }
    }

    pub fn exec(&mut self, pipeline: &PipelineDefinition) -> Result<(), Vec<String>> {
        let res = self.build(pipeline);

        match res {
            Err(err) => Err(vec![err]),
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
                    println!("app exec!");
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
                    Err(errors)
                }
            }
        }
    }

    fn build(&self, pipeline: &PipelineDefinition) -> Result<Vec<PipelineAction>, String> {
        pipeline
            .actions
            .iter()
            .map(|s| {
                if matches!(s.optional, Some(true) | None) {
                    match &s.value {
                        SelectionType::Single(a) => Ok(vec![a.clone()]),
                        SelectionType::OneOf(values, key) => values
                            .get(key)
                            .ok_or(format!("missing action {key}"))
                            .map(|a| vec![a.clone()]),
                        SelectionType::AnyOf(values, keys) => {
                            let mut ordered = keys
                                .iter()
                                .map(|k| {
                                    values
                                        .get_index_of(k)
                                        .map(|i| (i, k))
                                        .ok_or_else(|| format!("missing action {k}"))
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
    fn exec(&self, ctx: &mut Context, action: ActionType) -> Result<(), String> {
        match action {
            ActionType::Dependencies => {
                let ids = self.get_dependencies();

                let deps = ids
                    .iter()
                    .map(|id: &super::dependency::DependencyId| {
                        ctx.dependencies
                            .get(id)
                            .map(|d| (*d).clone())
                            .ok_or_else(|| format!("missing dependency {id:?}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

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
