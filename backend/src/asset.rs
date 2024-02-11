use anyhow::{Context, Result};
use include_dir::{Dir, File};
use std::{
    borrow::Cow,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::util::create_dir_all;

#[derive(Debug, Clone)]
pub struct AssetManager<'a> {
    embedded_assets: &'a Dir<'a>,
    external_asset_path: PathBuf,
    // TODO::in-memory assets (default templates)
}

impl<'a> AssetManager<'a> {
    pub fn new(embedded_assets: &'a Dir<'a>, external_asset_path: PathBuf) -> Self {
        Self {
            embedded_assets,
            external_asset_path,
        }
    }

    /// Retrieves an asset at [asset_path], where [asset_path] is relative to the base assets directory.
    ///
    /// # Example
    /// ```
    /// let asset = manager.get(PathBuf::from("kwin/emulatorwindowing.kwinscript"))
    /// ```
    pub fn get<'b, P: AsRef<Path> + std::fmt::Debug>(&'b self, asset_path: P) -> Option<Asset<'a>> {
        let external = self.external_asset_path.join(&asset_path);

        fn get_external(external: PathBuf) -> Result<Option<AssetType<'static>>> {
            Ok(if external.exists() && external.is_file() {
                Some(AssetType::External(external))
            } else {
                None
            })
        }

        get_external(external)
            .ok()
            .flatten()
            .or_else(|| {
                self.embedded_assets
                    .get_file(&asset_path)
                    .map(AssetType::Internal)
            })
            .map(|a| Asset { asset_impl: a })
    }
}

pub struct Asset<'a> {
    asset_impl: AssetType<'a>,
}

enum AssetType<'a> {
    Internal(&'a File<'a>),
    External(PathBuf),
}

impl<'a> Asset<'a> {
    pub fn file_path(&self) -> Result<PathBuf> {
        match &self.asset_impl {
            AssetType::Internal(file) => {
                // Since embedded files aren't "real" to the filesystem,
                // we copy the embedded file out to the tmp directory
                // so that the path may be used by external programs.
                let internal_path = file.path();
                let tmp_dir = std::env::temp_dir().join("DeckDS");

                let external_path = tmp_dir.join(internal_path);

                create_dir_all(
                    external_path
                        .parent()
                        .expect("external asset path should have a parent"),
                )
                .with_context(|| "failed to create dir to write bundled asset")?;

                log::info!("Writing bundled asset to {external_path:?}");

                std::fs::write(&external_path, file.contents())?;
                Ok(external_path)
            }
            AssetType::External(file) => Ok(file.to_path_buf()),
        }
    }

    pub fn contents(&self) -> Result<Cow<[u8]>> {
        Ok(match &self.asset_impl {
            AssetType::Internal(file) => Cow::Borrowed(file.contents()),
            AssetType::External(file) => Cow::Owned(std::fs::read(file)?),
        })
    }

    pub fn contents_to_string(&self) -> Result<Cow<str>, std::io::Error> {
        Ok(match &self.asset_impl {
            AssetType::Internal(file) => Cow::Borrowed(
                file.contents_utf8()
                    .ok_or::<std::io::Error>(ErrorKind::InvalidData.into())?,
            ),
            AssetType::External(file) => Cow::Owned(std::fs::read_to_string(file)?),
        })
    }
}
