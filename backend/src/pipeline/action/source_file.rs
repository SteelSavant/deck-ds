use std::path::{Path, PathBuf};

use anyhow::Context;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::{dependency::Dependency, executor::PipelineContext};

use super::{ActionId, ActionImpl};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct SourceFile {
    pub id: ActionId,
    pub source: FileSource,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum FileSource {
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
    // Don't really expect any other EmuDeck sources to depend on it, but hey.
}

impl SettingsSource for EmuDeckSource {
    fn settings_file(&self, ctx: &PipelineContext) -> PathBuf {
        let emudeck_settings_file = ctx.home_dir.join("emudeck/settings.sh");

        log::debug!("found emudeck settings file");

        let emudeck_settings = std::fs::read_to_string(&emudeck_settings_file);

        match emudeck_settings {
            Ok(emudeck_settings) => {
                log::debug!("emudeck settings: {emudeck_settings}");

                let rxp = Regex::new(r"emulationPath=(.*)")
                    .expect("emudeck emulation path regex should be valid");
                let found = rxp.captures(&emudeck_settings);

                log::debug!("found emudeck captures {found:?}");

                match found {
                    Some(c) => {
                        let path = Path::new(c.get(1).unwrap().as_str().trim());
                        let resolved = if !path.is_dir() && ctx.home_dir.join(path).is_dir() {
                            ctx.home_dir.join(path)
                        } else {
                            path.to_path_buf()
                        };

                        let cemu_proton_path = resolved.join("roms/wiiu/settings.xml");
                        log::debug!("cemu_proton_path at {cemu_proton_path:?}",);

                        match self {
                            EmuDeckSource::CemuProton => cemu_proton_path,
                        }
                    }
                    None => emudeck_settings_file, // if this is missing, nothing works, so we return it as the path for deps to tell us its missing/wrong
                }
            }
            Err(_) => emudeck_settings_file, // same reasoning as previous return
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

    const NAME: &'static str = "SourceFile";

    fn setup(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        match &self.source {
            FileSource::Flatpak(flatpak) => {
                ctx.set_state::<Self>(flatpak.settings_file(ctx));

                Ok(())
            }
            FileSource::AppImage(appimage) => {
                ctx.set_state::<Self>(appimage.settings_file(ctx));

                Ok(())
            }
            FileSource::EmuDeck(emudeck) => {
                ctx.set_state::<Self>(emudeck.settings_file(ctx));

                Ok(())
            }
            FileSource::Custom(CustomFileOptions {
                path: Some(file), ..
            }) => {
                ctx.set_state::<Self>(file.clone());

                Ok(())
            }
            FileSource::Custom(CustomFileOptions { path: None, .. }) => {
                None.with_context(|| "could not set source file; field not set")
            }
        }
    }

    fn get_dependencies(
        &self,
        ctx: &mut PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        let dep = match &self.source {
            FileSource::Flatpak(flatpak) => Dependency::Path {
                path: flatpak.settings_file(ctx),
                is_file: true,
            },
            FileSource::AppImage(appimage) => Dependency::Path {
                path: appimage.settings_file(ctx),
                is_file: true,
            },
            FileSource::EmuDeck(emudeck) => Dependency::Path {
                path: emudeck.settings_file(ctx),
                is_file: true,
            },
            FileSource::Custom(CustomFileOptions {
                path: Some(file), ..
            }) => Dependency::Path {
                path: file.clone(),
                is_file: true,
            },
            FileSource::Custom(CustomFileOptions { path: None, .. }) => {
                Dependency::ConfigField("File Path".to_string())
            }
        };
        vec![dep]
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
        let expected = FileSource::Custom(CustomFileOptions {
            valid_ext: vec![".ini".to_string()],
            path: None,
        });
        let json = serde_json::to_string(&expected)?;
        let actual = serde_json::from_str(&json)?;
        assert_eq!(expected, actual);

        Ok(())
    }
}
