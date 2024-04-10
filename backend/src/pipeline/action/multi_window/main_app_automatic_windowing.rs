use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::action::{ActionId, ActionImpl, ActionType},
    sys::windowing::get_window_info_from_pid_default_active_after,
    util::escape_string_for_regex,
};

use super::primary_windowing::{CustomWindowOptions, GeneralOptions, MultiWindow};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MainAppAutomaticWindowing {
    pub id: ActionId,
    pub general: GeneralOptions,
}

impl ActionImpl for MainAppAutomaticWindowing {
    type State = MultiWindow;

    const TYPE: ActionType = ActionType::MainAppAutomaticWindowing;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let id = self.id;
        let general = self.general.clone();

        ctx.register_on_launch_callback(Box::new(move |pid, ctx| {
            let info = get_window_info_from_pid_default_active_after(pid, Duration::from_secs(5))?;
            let multi = MultiWindow {
                id: id,
                general: general.clone(),
                cemu: None,
                citra: None,
                dolphin: None,
                custom: Some(CustomWindowOptions {
                    primary_window_matcher: Some(escape_string_for_regex(info.name)),
                    secondary_window_matcher: None,
                    classes: info.classes,
                    ..Default::default()
                }),
            };
            multi.setup(ctx)?;
            ctx.set_state::<Self>(multi);
            Ok(())
        }));

        Ok(())
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        if let Some(multi) = ctx.get_state::<Self>().cloned() {
            multi.teardown(ctx)
        } else {
            Ok(())
        }
    }

    fn get_dependencies(
        &self,
        ctx: &crate::pipeline::executor::PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        MultiWindow {
            id: ActionId::nil(),
            general: GeneralOptions::default(),
            cemu: None,
            citra: None,
            dolphin: None,
            custom: None,
        }
        .get_dependencies(ctx)
    }
}
