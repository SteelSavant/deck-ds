use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pipeline::{dependency::Dependency, executor::PipelineContext};

use super::super::{ActionId, ActionImpl, ActionType};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct EmuSettingsSourceConfig {
    pub id: ActionId,
    pub source: EmuSettingsSource,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
// enum_dispatch settings_file here...
pub enum EmuSettingsSource {
    Flatpak(FlatpakSource),
    AppImage(AppImageSource),
    EmuDeck(EmuDeckSource),
    Custom(CustomEmuSource),
}

trait EmuSettingsSourceFile {
    fn settings_file(&self, ctx: &PipelineContext) -> Result<PathBuf, EmuSettingsSourceFileError>;
}

#[derive(Error, Debug)]
pub enum EmuSettingsSourceFileError {
    #[error("Emudeck settings not found at {0}")]
    MissingEmudeckSettings(PathBuf),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct CustomEmuSource {
    /// valid file extensions for settings file
    pub valid_ext: Vec<String>,
    /// command to run the emulator
    // pub emu_cmd: Option<PathBuf>,
    /// user defined custom path
    pub settings_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FlatpakSource {
    Cemu,
    Citra,
    MelonDS,
    Lime3ds,
}

impl FlatpakSource {
    fn org(&self) -> &'static str {
        match self {
            FlatpakSource::Cemu => "info.cemu.Cemu",
            FlatpakSource::Citra => "org.citra_emu.citra",
            FlatpakSource::MelonDS => "net.kuribo64.melonDS",
            FlatpakSource::Lime3ds => "io.github.lime3ds.Lime3DS",
        }
    }
}

impl EmuSettingsSourceFile for FlatpakSource {
    fn settings_file(&self, ctx: &PipelineContext) -> Result<PathBuf, EmuSettingsSourceFileError> {
        let dir = ctx
            .decky_env
            .deck_user_home
            .join(".var/app")
            .join(self.org());
        let res = match self {
            FlatpakSource::Cemu => dir.join("config/Cemu/settings.xml"),
            FlatpakSource::Citra => dir.join("config/citra-emu/qt-config.ini"),
            FlatpakSource::MelonDS => dir.join("config/melonDS/melonDS.ini"),
            FlatpakSource::Lime3ds => dir.join("config/citra-emu/qt-config.ini"),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EmuDeckSource {
    CemuProton,
    // Don't really expect any other EmuDeck sources to depend on it, but hey.
}

impl EmuSettingsSourceFile for EmuDeckSource {
    fn settings_file(&self, ctx: &PipelineContext) -> Result<PathBuf, EmuSettingsSourceFileError> {
        let emudeck_settings_file = ctx.decky_env.deck_user_home.join("emudeck/settings.sh");

        log::debug!("found emudeck settings file");

        let emudeck_settings = std::fs::read_to_string(&emudeck_settings_file);

        let err = Err(EmuSettingsSourceFileError::MissingEmudeckSettings(
            emudeck_settings_file,
        ));

        match emudeck_settings {
            Ok(emudeck_settings) => {
                let rxp = Regex::new(r"emulationPath=(.*)")
                    .expect("emudeck emulation path regex should be valid");
                let found = rxp.captures(&emudeck_settings);

                log::debug!("found emudeck captures {found:?}");

                match found {
                    Some(c) => {
                        let path = Path::new(c.get(1).unwrap().as_str().trim());
                        let resolved =
                            if !path.is_dir() && ctx.decky_env.deck_user_home.join(path).is_dir() {
                                ctx.decky_env.deck_user_home.join(path)
                            } else {
                                path.to_path_buf()
                            };

                        let cemu_proton_path = resolved.join("roms/wiiu/settings.xml");
                        log::debug!("cemu_proton_path at {cemu_proton_path:?}",);

                        match self {
                            EmuDeckSource::CemuProton => Ok(cemu_proton_path),
                        }
                    }
                    None => err,
                }
            }
            Err(_) => err,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AppImageSource {
    Cemu,
}

impl EmuSettingsSourceFile for AppImageSource {
    fn settings_file(&self, ctx: &PipelineContext) -> Result<PathBuf, EmuSettingsSourceFileError> {
        let res = match self {
            AppImageSource::Cemu => ctx
                .decky_env
                .deck_user_home
                .join(".config/Cemu/settings.xml"),
        };

        Ok(res)
    }
}

impl ActionImpl for EmuSettingsSourceConfig {
    type State = PathBuf;

    const TYPE: ActionType = ActionType::SourceFile;

    fn setup(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        match &self.source {
            EmuSettingsSource::Flatpak(flatpak) => {
                ctx.set_state::<Self>(flatpak.settings_file(ctx)?);

                Ok(())
            }
            EmuSettingsSource::AppImage(appimage) => {
                ctx.set_state::<Self>(appimage.settings_file(ctx)?);

                Ok(())
            }
            EmuSettingsSource::EmuDeck(emudeck) => {
                ctx.set_state::<Self>(emudeck.settings_file(ctx)?);

                Ok(())
            }
            EmuSettingsSource::Custom(CustomEmuSource {
                settings_path: Some(file),
                ..
            }) => {
                ctx.set_state::<Self>(file.clone());

                Ok(())
            }
            EmuSettingsSource::Custom(CustomEmuSource {
                settings_path: None,
                ..
            }) => anyhow::bail!("could not set source file; field not set"),
        }
    }

    fn get_dependencies(
        &self,
        ctx: &PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        fn get_settings_file_path(
            this: &EmuSettingsSourceConfig,
            ctx: &PipelineContext,
        ) -> Result<Dependency, EmuSettingsSourceFileError> {
            Ok(match &this.source {
                EmuSettingsSource::Flatpak(flatpak) => Dependency::Path {
                    path: flatpak.settings_file(ctx)?,
                    is_file: true,
                },
                EmuSettingsSource::AppImage(appimage) => Dependency::Path {
                    path: appimage.settings_file(ctx)?,
                    is_file: true,
                },
                EmuSettingsSource::EmuDeck(emudeck) => Dependency::Path {
                    path: emudeck.settings_file(ctx)?,
                    is_file: true,
                },
                EmuSettingsSource::Custom(CustomEmuSource {
                    settings_path: Some(file),
                    ..
                }) => Dependency::Path {
                    path: file.clone(),
                    is_file: true,
                },
                EmuSettingsSource::Custom(CustomEmuSource {
                    settings_path: None,
                    ..
                }) => Dependency::ConfigField("File Path".to_string()),
            })
        }

        let dep = get_settings_file_path(self, ctx);
        match dep {
            Ok(dep) => vec![dep],
            Err(err) => match err {
                EmuSettingsSourceFileError::MissingEmudeckSettings(err) => {
                    vec![Dependency::EmuDeckSettings(err)]
                }
            },
        }
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_custom_serde() -> Result<()> {
        let expected = EmuSettingsSource::Custom(CustomEmuSource {
            valid_ext: vec![".ini".to_string()],
            settings_path: None,
            // emu_cmd: None,
        });
        let json = serde_json::to_string(&expected)?;
        let actual = serde_json::from_str(&json)?;
        assert_eq!(expected, actual);

        Ok(())
    }
}

// If "Versioned" is a Selection, how does it determine the source app to query?
// - track actions in tree during descent, pick EmuSource above?
// - require Versioned to take a PipelineActionId to an EmuSource?
// If "Versioned" is part of an action (like EmuSource), how is the correct action resolved during reification?
