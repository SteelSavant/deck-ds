use std::path::PathBuf;

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::{dependency::Dependency, executor::PipelineContext};

use super::ActionImpl;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum SourceFile {
    Flatpak(FlatpakSource),
    AppImage(AppImageSource),
    EmuDeck(EmuDeckSource),
    Custom(CustomFileOptions),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct CustomFileOptions {
    /// valid file extensions for source file
    pub valid_ext: Vec<String>,
    /// user defined custom path
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FlatpakSource {
    Cemu,
    Citra,
    MelonDS,
}

trait SettingsSource {
    fn settings_file(&self, ctx: &PipelineContext) -> PathBuf;
}

impl FlatpakSource {
    fn org(&self) -> &'static str {
        match self {
            FlatpakSource::Cemu => "info.cemu.Cemu",
            FlatpakSource::Citra => "org.citra_emu.citra",
            FlatpakSource::MelonDS => "net.kuribo64.melonDS",
        }
    }
}

impl SettingsSource for FlatpakSource {
    fn settings_file(&self, ctx: &PipelineContext) -> PathBuf {
        let dir = ctx.home_dir.join(".var/app").join(self.org());
        match self {
            FlatpakSource::Cemu => dir.join("config/Cemu/settings.xml"),
            FlatpakSource::Citra => dir.join("config/citra-emu/qt-config.ini"),
            FlatpakSource::MelonDS => dir.join("config/melonDS/melonDS.ini"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EmuDeckSource {
    CemuProton,
}

impl SettingsSource for EmuDeckSource {
    fn settings_file(&self, ctx: &PipelineContext) -> PathBuf {
        match self {
            EmuDeckSource::CemuProton => ctx.home_dir.join("Emulation/roms/wiiu/settings.xml"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AppImageSource {
    Cemu,
}

impl SettingsSource for AppImageSource {
    fn settings_file(&self, ctx: &PipelineContext) -> PathBuf {
        match self {
            AppImageSource::Cemu => ctx.config_dir.join("Cemu/settings.xml"),
        }
    }
}

impl ActionImpl for SourceFile {
    type State = PathBuf;

    fn setup(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        match &self {
            SourceFile::Flatpak(flatpak) => {
                ctx.set_state::<Self>(flatpak.settings_file(ctx));

                Ok(())
            }
            SourceFile::AppImage(appimage) => {
                ctx.set_state::<Self>(appimage.settings_file(ctx));

                Ok(())
            }
            SourceFile::EmuDeck(emudeck) => {
                ctx.set_state::<Self>(emudeck.settings_file(ctx));

                Ok(())
            }
            SourceFile::Custom(CustomFileOptions {
                path: Some(file), ..
            }) => {
                ctx.set_state::<Self>(file.clone());

                Ok(())
            }
            SourceFile::Custom(CustomFileOptions { path: None, .. }) => {
                None.with_context(|| "could not set source file; field not set")
            }
        }
    }

    fn get_dependencies(
        &self,
        ctx: &mut PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        let dep = match &self {
            SourceFile::Flatpak(flatpak) => Dependency::Path {
                path: flatpak.settings_file(ctx),
                is_file: true,
            },
            SourceFile::AppImage(appimage) => Dependency::Path {
                path: appimage.settings_file(ctx),
                is_file: true,
            },
            SourceFile::EmuDeck(emudeck) => Dependency::Path {
                path: emudeck.settings_file(ctx),
                is_file: true,
            },
            SourceFile::Custom(CustomFileOptions {
                path: Some(file), ..
            }) => Dependency::Path {
                path: file.clone(),
                is_file: true,
            },
            SourceFile::Custom(CustomFileOptions { path: None, .. }) => {
                Dependency::FieldNotSet("Custom File".to_string())
            }
        };
        vec![dep]
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_custom_serde() -> Result<()> {
        let expected = SourceFile::Custom(CustomFileOptions {
            valid_ext: vec![".ini".to_string()],
            path: None,
        });
        let json = serde_json::to_string(&expected)?;
        let actual = serde_json::from_str(&json)?;
        assert_eq!(expected, actual);

        Ok(())
    }
}
