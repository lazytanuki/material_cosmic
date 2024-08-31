use std::path::Path;

use anyhow::Context;
use cosmic_config::CosmicConfigEntry;
use cosmic_settings_daemon::CosmicSettingsDaemonProxy;
use cosmic_theme::{Theme, ThemeBuilder};
use palette::{cam16::IntoCam16Unclamped, Darken};
use wallust::colors::Colors;
use zbus::Connection;

pub mod config;
pub mod options;

/// Generate colors using wallust, with dynamic threshold.
pub fn generate_colors(
    wallpaper_path: impl AsRef<Path>,
    wallust_config: &wallust::config::Config,
    cache_path: &Path,
    overwrite_cache: bool,
) -> anyhow::Result<Colors> {
    log::info!(
        "generating colors using wallust and backend {:?}",
        wallust_config.backend_user
    );

    // Generate hash cache file name and cache dir to either read or write to it
    let mut cached_data =
        wallust::cache::Cache::new(wallpaper_path.as_ref(), wallust_config, cache_path)
            .with_context(|| "unable to create cache")?;

    // Directly return cached colors, if any
    if !overwrite_cache && cached_data.is_cached() {
        match cached_data.read() {
            Ok(data) => return Ok(data),
            Err(e) => {
                log::error!("unable to read cached data, continuing without it ({})", e);
            }
        }
    }

    // Otherwise, generate colors
    let colors = wallust::gen_colors(wallpaper_path.as_ref(), wallust_config, false)
        .with_context(|| "unable to generate colors using wallust")?
        .0;

    // Write newly generated colors to cache
    if let Err(e) = cached_data.write(&colors) {
        log::error!("unable to write newly generated colors to cache ({e})");
    }

    Ok(colors)
}

/// Apply generated colors to terminals by sending them sequences
pub fn apply_colors_to_terminals(colors: &Colors, cache_path: &Path) -> anyhow::Result<()> {
    colors.sequences(cache_path, None)
}

/// Set colors to COSMIC desktop
pub async fn apply_colors_to_desktop<'a>(
    colors: &Colors,
    // settings_proxy: &'a CosmicSettingsDaemonProxy<'a>,
    is_dark: bool,
) -> anyhow::Result<()> {
    // Connect to the settings daemon
    // Retrieve default theme and apply colors to it
    let (builder_config, default) = if is_dark {
        (ThemeBuilder::dark_config()?, Theme::dark_default())
    } else {
        (ThemeBuilder::light_config()?, Theme::light_default())
    };

    let mut theme = match ThemeBuilder::get_entry(&builder_config) {
        Ok(entry) => entry,
        Err((errs, entry)) => {
            for err in errs {
                log::error!("failed to get theme: {}", err);
            }
            entry
        }
    };

    theme.write_entry(&builder_config)?;
    theme = theme.accent(colors.color5.0);
    theme = theme.bg_color(colors.color1.0.darken(0.3).into());
    theme = theme.text_tint(colors.color6.0);
    theme = theme.neutral_tint(colors.background.0);
    let theme = theme.build();
    let theme_config = if theme.is_dark {
        Theme::dark_config()
    } else {
        Theme::light_config()
    }?;

    theme.write_entry(&theme_config)?;

    Ok(())
}

/// Get connection to session bus
async fn load_conn() -> anyhow::Result<Connection> {
    for _ in 0..5 {
        match Connection::session().await {
            Ok(conn) => return Ok(conn),
            Err(e) => {
                log::error!("failed to connect to the session bus: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    Err(anyhow::anyhow!("failed to connect to the session bus"))
}

/// Connect to the settings daemon through the session bus
async fn connect_settings_daemon() -> anyhow::Result<CosmicSettingsDaemonProxy<'static>> {
    let conn = load_conn().await?;
    for _ in 0..5 {
        match CosmicSettingsDaemonProxy::builder(&conn).build().await {
            Ok(proxy) => return Ok(proxy),
            Err(e) => {
                log::error!("Failed to connect to the settings daemon: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    Err(anyhow::anyhow!("Failed to connect to the settings daemon"))
}
