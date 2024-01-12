use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

use crate::{native_model_serde_json::NativeModelJSON, settings::AppId};

pub type DbCategoryProfile = v1::DbCategoryProfile;

pub mod v1 {
    use crate::{
        pipeline::{action::ActionId, data::generic},
        settings::ProfileId,
    };

    use super::*;

    #[derive(Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 1, version = 1, with = NativeModelJSON)]

    pub struct DbCategoryProfile {
        #[primary_key]
        pub id: ProfileId,
        pub tags: Vec<String>,
        pub pipeline: DbPipelineDefinition,
    }

    #[derive(Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 2, version = 1, with = NativeModelJSON)]
    pub struct DbAppProfile {
        #[primary_key]
        pub id: AppId,
    }

    pub type DbPipelineDefinition = generic::PipelineDefinition<DbAction>;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum DbAction {
        DisplayRestoration(ActionId),
        VirtualScreen(ActionId),
        MultiWindow(ActionId),
        CitraLayout(ActionId),
        CemuLayout(ActionId),
        MelonDSLayout(ActionId),
        SourceFile(ActionId),
    }
}
