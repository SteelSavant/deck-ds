use std::time::Duration;

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::action::{Action, ActionId, ActionImpl, ActionType, ErasedPipelineAction},
    settings::SteamLaunchInfo,
    sys::kwin::KWinClientMatcher,
    util::{escape_string_for_regex, get_maybe_window_names_classes_from_title},
};

use super::primary_windowing::{CustomWindowOptions, GeneralOptions, MultiWindow};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MainAppAutomaticWindowing {
    pub id: ActionId,
    pub general: GeneralOptions,
}

impl ActionImpl for MainAppAutomaticWindowing {
    type State = Action;

    const TYPE: ActionType = ActionType::MainAppAutomaticWindowing;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let id = self.id;
        let general = self.general.clone();

        let window_ctx = ctx.kwin.start_tracking_new_windows()?;

        let launch_info = ctx
            .launch_info
            .as_ref()
            .expect("main app automatic windowing requires launch info");

        let maybe_strings = get_maybe_window_names_from_launch_info(&launch_info);

        ctx.register_on_launch_callback(Box::new(move |_pid, ctx| {
            log::debug!("main app automatic windowing callback");

            let best_window = window_ctx
                .get_best_window_client(KWinClientMatcher {
                    min_delay: Duration::from_secs(5),
                    max_delay: Duration::from_secs(30),
                    preferred_ord_if_no_match: std::cmp::Ordering::Greater,
                    maybe_strings, // match_fn: Box::new(move |clients| {
                                   //     maybe_strings;
                                   //     clients.into_iter().last().cloned()
                                   // }),
                })?
                .context("automatic windowing expected to find a window")?;

            log::debug!("using {best_window:?} as app window");

            let multi = Action::from(MultiWindow {
                id,
                general: general.clone(),
                cemu: None,
                citra: None,
                dolphin: None,
                custom: Some(CustomWindowOptions {
                    primary_window_matcher: Some(escape_string_for_regex(best_window.caption)),
                    secondary_window_matcher: None,
                    classes: best_window.window_classes,
                    ..Default::default()
                }),
            });
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
        ActionImpl::get_dependencies(
            &MultiWindow {
                id: ActionId::nil(),
                general: GeneralOptions::default(),
                cemu: None,
                citra: None,
                dolphin: None,
                custom: None,
            },
            ctx,
        )
    }
}

fn get_maybe_window_names_from_launch_info(launch_info: &SteamLaunchInfo) -> Vec<String> {
    let mut maybes = get_maybe_window_names_classes_from_title(&launch_info.game_title);
    maybes.push(format!("steam_app_{}", launch_info.app_id.raw()));

    maybes
}

// Totally Broken:
// - Nidhogg (closes immediately)
// - Ultimate Chicken Horse (closes immediately)
// - Peggle (horrible flickering, wrong window size)
// - Broforce (closes immediately)
// - Castle Crashers (closes immediately)

// Broken (Fixable)
// - Lovers in a Dangerous Spacetime (wrong window size) [Fixable in game settings]

// Work:
// - Everspace (questionable resolution)
// - One Step From Eden
