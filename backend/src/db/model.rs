use native_db::DatabaseBuilder;
use once_cell::sync::Lazy;
use strum::IntoEnumIterator;

use crate::pipeline::action::ActionType;

mod v1;

// Core

pub type DbCategoryProfile = v1::DbCategoryProfile;
pub type DbAppOverride = v1::DbAppOverride;
pub type DbAppSettings = v1::DbAppSettings;
pub type DbPipelineDefinition = v1::DbPipelineDefinition;
pub type DbTopLevelDefinition = v1::DbTopLevelDefinition;
pub type DbConfigSelection = v1::DbConfigSelection;
pub type DbAction = v1::DbAction;
pub type DbPipelineActionSettings = v1::DbPipelineActionSettings;
pub type DbExitHooks = v1::DbExitHooks;

// Action

pub type DbCemuLayout = v1::DbCemuLayout;
pub type DbCemuAudio = v1::DbCemuAudio;
pub type DbCitraLayout = v1::DbCitraLayout;
pub type DbLime3dsLayout = v1::DbLime3dsLayout;
pub type DbMelonDSLayout = v1::DbMelonDSLayout;
pub type DbDesktopSessionHandler = v1::DbDesktopSessionHandler;
pub type DbMultiWindow = v1::DbMultiWindow;
pub type DbSourceFile = v1::DbSourceFile;
pub type DbVirtualScreen = v1::DbVirtualScreen;
pub type DbDisplayConfig = v1::DbDisplayConfig;
pub type DbLaunchSecondaryApp = v1::DbLaunchSecondaryFlatpakApp;
pub type DbLaunchSecondaryAppPreset = v1::DbLaunchSecondaryAppPreset;
pub type DbMainAppAutomaticWindowing = v1::DbMainAppAutomaticWindowing;

pub static DATABASE_BUILDER: Lazy<native_db::DatabaseBuilder> = Lazy::new(|| {
    let mut builder = DatabaseBuilder::new();
    // V1
    {
        // Profiles

        builder
            .define::<v1::DbCategoryProfile>()
            .expect("failed to define CategoryProfile v1");
        builder
            .define::<v1::DbAppSettings>()
            .expect("failed to define AppProfile v1");
        builder
            .define::<v1::DbAppOverride>()
            .expect("failed to define AppProfile v1");
        builder
            .define::<v1::DbPipelineDefinition>()
            .expect("failed to define DbPipelineDefinition v1");
        builder
            .define::<v1::DbPipelineActionSettings>()
            .expect("failed to define DbPipelineActionSettings v1");

        // Actions

        let v1_actions = [
            builder
                .define::<v1::DbCemuLayout>()
                .expect("failed to define CemuLayout v1"),
            builder
                .define::<v1::DbCemuAudio>()
                .expect("failed to define DbCemuAudio v1"),
            builder
                .define::<v1::DbCitraLayout>()
                .expect("failed to define CitraLayout v1"),
            builder
                .define::<v1::DbLime3dsLayout>()
                .expect("failed to define DbLime3dsLayout v1"),
            builder
                .define::<v1::DbMelonDSLayout>()
                .expect("failed to define MelonDSLayout v1"),
            builder
                .define::<v1::DbDesktopSessionHandler>()
                .expect("failed to define DesktopSessionHandler v1"),
            builder
                .define::<v1::DbMultiWindow>()
                .expect("failed to define MultiWindow v1"),
            builder
                .define::<v1::DbVirtualScreen>()
                .expect("failed to define VirtualScreen v1"),
            builder
                .define::<v1::DbSourceFile>()
                .expect("failed to define SourceDbSourceFile v1"),
            builder
                .define::<v1::DbDisplayConfig>()
                .expect("failed to define DbDisplayConfig v1"),
            builder
                .define::<v1::DbLaunchSecondaryFlatpakApp>()
                .expect("failed to define LaunchSecondaryApp v1"),
            builder
                .define::<v1::DbLaunchSecondaryAppPreset>()
                .expect("failed to define LaunchSecondaryAppPreset v1"),
            builder
                .define::<v1::DbMainAppAutomaticWindowing>()
                .expect("failed to define DbMainAppAutomaticWindowing  v1"),
        ];

        assert_eq!(ActionType::iter().len(), v1_actions.len());
    }

    builder
});
