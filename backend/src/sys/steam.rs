use std::path::{Path, PathBuf};

use crate::settings::{AppId, UserId};
use anyhow::{Context, Result};

pub fn set_desktop_controller_hack<P: AsRef<Path>>(
    user: &UserId,
    app_id: &AppId,
    game_title: &str,
    steam_dir: P,
) -> Result<()> {
    let desktop_layout_path = get_desktop_layout_path(user, steam_dir.as_ref());
    let desktop_layout_backup_path = desktop_layout_path.with_extension("vdf.bck");

    if !desktop_layout_backup_path.exists() {
        std::fs::copy(&desktop_layout_path, &desktop_layout_backup_path)
            .with_context(|| "failed to create backup desktop controller config file")?;
    }

    let best_controller_folder =
        get_best_game_folder(user, app_id, game_title, steam_dir.as_ref())?;
    match best_controller_folder {
        Some(folder) => std::fs::copy(folder.join("controller_neptune.vdf"), desktop_layout_path)
            .map(|_| ())
            .with_context(|| "failed to copy controller config file"),
        None => {
            log::warn!(
                "no config file found for {:?}: {} to copy",
                app_id,
                game_title
            );
            Ok(())
        }
    }
}

pub fn unset_desktop_controller_hack<P: AsRef<Path>>(steam_dir: P) -> Result<()> {
    for entry in get_configs_dir(steam_dir.as_ref())
        .read_dir()?
        .filter_map(|v| v.ok())
    {
        let user = UserId::new(&entry.file_name().to_string_lossy());
        let desktop_layout_path = get_desktop_layout_path(&user, &steam_dir);
        let desktop_layout_backup_path = desktop_layout_path.with_extension("vdf.bck");

        if desktop_layout_backup_path.exists() {
            let res = std::fs::copy(&desktop_layout_backup_path, &desktop_layout_path)
                .with_context(|| "failed to restore backup desktop controller config file")
                .and_then(|_| {
                    std::fs::remove_file(desktop_layout_backup_path)
                        .with_context(|| "failed to remove backup desktop controller config file")
                });

            if let Err(err) = res {
                log::warn!("{}", err);
            }
        }
    }

    Ok(())
}

fn get_best_game_folder<P: AsRef<Path>>(
    user: &UserId,
    app_id: &AppId,
    game_title: &str,
    steam_dir: P,
) -> Result<Option<PathBuf>> {
    use str_distance::*;

    get_layout_dir(user, steam_dir)
        .read_dir()
        .map(|v| {
            v.filter_map(|v| v.ok())
                .filter(|v| v.path().is_dir())
                .fold((1., None), |acc, next| {
                    if next.file_name() == app_id.raw() {
                        // If the folder matches the app id, its the desired folder
                        return (0., Some(next.path()));
                    } else {
                        // Otherwise, take the best-matching non-steam config folder.
                        const TITLE_THRESH: f64 = 0.2;
                        let title_distance = str_distance_normalized(
                            next.file_name().to_string_lossy(),
                            game_title,
                            Levenshtein::default(),
                        );
                        if title_distance < TITLE_THRESH && title_distance < acc.0 {
                            return (title_distance, Some(next.path()));
                        } else {
                            acc
                        }
                    }
                })
                .1
        })
        .with_context(|| "failed to search controller config dir for best layout")
}

fn get_desktop_layout_path<P: AsRef<Path>>(user: &UserId, steam_dir: P) -> PathBuf {
    get_layout_path(user, "47870", steam_dir)
}

fn get_layout_path<P: AsRef<Path>>(user: &UserId, config_dir: &str, steam_dir: P) -> PathBuf {
    get_layout_dir(user, steam_dir)
        .join(config_dir)
        .join("controller_neptune.vdf")
}

fn get_layout_dir<P: AsRef<Path>>(user: &UserId, steam_dir: P) -> PathBuf {
    get_configs_dir(steam_dir).join(user.raw()).join("config")
}

fn get_configs_dir<P: AsRef<Path>>(steam_dir: P) -> PathBuf {
    steam_dir
        .as_ref()
        .join("steamapps")
        .join("common")
        .join("Steam Controller Configs")
}

#[cfg(test)]
mod tests {
    use crate::util::create_dir_all;

    use super::*;

    fn get_user() -> UserId {
        UserId::new("1000")
    }

    fn get_appid() -> AppId {
        AppId::new("99999")
    }

    fn setup_dir(steam_dir: &str) {
        let steam_dir = Path::new(steam_dir);
        let _ = std::fs::remove_dir_all(steam_dir);

        let user = get_user();

        let desktop_layout_path = get_desktop_layout_path(&user, steam_dir);

        create_dir_all(&desktop_layout_path.parent().unwrap()).unwrap();

        let configs_folder = desktop_layout_path.parent().unwrap().parent().unwrap();
        let steam_path = configs_folder.join("99999/controller_neptune.vdf");
        create_dir_all(&steam_path.parent().unwrap()).unwrap();

        let nonsteam_path = configs_folder.join("nonsteam/controller_neptune.vdf");
        create_dir_all(&nonsteam_path.parent().unwrap()).unwrap();

        std::fs::write(desktop_layout_path, "desktop").unwrap();
        std::fs::write(steam_path, "steam").unwrap();
        std::fs::write(nonsteam_path, "nonsteam").unwrap();
    }

    #[test]
    fn test_set_controller_hack_steam() {
        let steam_dir = "./test/out/steam/set_steam/steam/";
        setup_dir(steam_dir);
        let user = get_user();
        let app_id = get_appid();
        set_desktop_controller_hack(&user, &app_id, "some title", steam_dir).unwrap();

        let desktop_controller_path = get_desktop_layout_path(&user, steam_dir);
        let backup = desktop_controller_path.with_extension("vdf.bck");

        let desktop_contents = std::fs::read_to_string(desktop_controller_path).unwrap();
        let desktop_backup_contents = std::fs::read_to_string(backup).unwrap();

        let nonsteam_contents =
            std::fs::read_to_string(&get_layout_path(&user, "nonsteam", steam_dir)).unwrap();
        let steam_contents =
            std::fs::read_to_string(&get_layout_path(&user, &app_id.raw(), steam_dir)).unwrap();

        assert_eq!("steam", &desktop_contents);
        assert_eq!("desktop", &desktop_backup_contents);
        assert_eq!("nonsteam", &nonsteam_contents);
        assert_eq!("steam", &steam_contents);
    }

    #[test]
    fn test_set_controller_hack_nonsteam() {
        let steam_dir = "./test/out/steam/set_nonsteam/steam";
        setup_dir(steam_dir);

        let user = get_user();
        let app_id = AppId::new("00000");
        set_desktop_controller_hack(&user, &app_id, "nonsteam", steam_dir).unwrap();

        let desktop_controller_path = get_desktop_layout_path(&user, steam_dir);
        let backup = desktop_controller_path.with_extension("vdf.bck");

        let desktop_contents = std::fs::read_to_string(desktop_controller_path).unwrap();
        let desktop_backup_contents = std::fs::read_to_string(backup).unwrap();

        let nonsteam_contents =
            std::fs::read_to_string(&get_layout_path(&user, "nonsteam", steam_dir)).unwrap();
        let steam_contents =
            std::fs::read_to_string(&get_layout_path(&user, &get_appid().raw(), steam_dir))
                .unwrap();

        assert_eq!("nonsteam", &desktop_contents);
        assert_eq!("desktop", &desktop_backup_contents);
        assert_eq!("nonsteam", &nonsteam_contents);
        assert_eq!("steam", &steam_contents);
    }

    #[test]
    fn test_unset_controller_hack() {
        let steam_dir = "./test/out/steam/unset/steam";
        setup_dir(steam_dir);
        let user = get_user();
        let appid = get_appid();
        set_desktop_controller_hack(&user, &appid, "nonsteam", steam_dir).unwrap();

        let desktop_controller_path = get_desktop_layout_path(&user, steam_dir);
        let backup = desktop_controller_path.with_extension("vdf.bck");

        unset_desktop_controller_hack(steam_dir).unwrap();

        let desktop_contents = std::fs::read_to_string(desktop_controller_path).unwrap();
        let nonsteam_contents =
            std::fs::read_to_string(&get_layout_path(&user, "nonsteam", steam_dir)).unwrap();
        let steam_contents =
            std::fs::read_to_string(&get_layout_path(&user, &appid.raw(), steam_dir)).unwrap();

        assert!(!backup.exists());
        assert_eq!("desktop", &desktop_contents);
        assert_eq!("nonsteam", &nonsteam_contents);
        assert_eq!("steam", &steam_contents);
    }
}