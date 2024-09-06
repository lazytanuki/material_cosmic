use std::{borrow::BorrowMut, path::Path};

use anyhow::Context;
use cosmic_config::CosmicConfigEntry;
use cosmic_settings_daemon::CosmicSettingsDaemonProxy;
use material_colors::{
    color::Argb,
    image::{FilterType, ImageReader},
    theme::{Theme, ThemeBuilder},
};
use palette::{cam16::IntoCam16Unclamped, Darken, IntoColor, RgbHue, Srgb, Srgba};
use wallust::colors::Colors;
use zbus::Connection;

pub mod config;
pub mod options;

/// Generate colors using wallust, with dynamic threshold.
pub fn generate_colors(
    wallpaper_path: impl AsRef<Path>,
    cache_path: &Path,
    overwrite_cache: bool,
) -> anyhow::Result<material_colors::theme::Theme> {
    let image = std::fs::read(wallpaper_path).with_context(|| "unable to read file")?;

    let mut data = ImageReader::read(image).expect("failed to read image");

    // Lancsoz3 takes a little longer, but provides the best pixels for color extraction.
    // However, if you don't like the results, you can always try other FilterType values.
    data.resize(128, 128, FilterType::Lanczos3);

    let theme = ThemeBuilder::with_source(ImageReader::extract_color(&data)).build();

    Ok(theme)
}

/// Apply generated colors to terminals by sending them sequences
pub fn apply_colors_to_terminals(colors: &Colors, cache_path: &Path) -> anyhow::Result<()> {
    colors.sequences(cache_path, None)
}

fn argb_to_srgba(argb: Argb) -> Srgba {
    Srgba::new(
        argb.red as f32 / 255.0,
        argb.green as f32 / 255.0,
        argb.blue as f32 / 255.0,
        argb.alpha as f32 / 255.0,
    )
}

/// Set colors to COSMIC desktop
pub async fn apply_colors_to_desktop<'a>(
    material_theme: &Theme,
    // settings_proxy: &'a CosmicSettingsDaemonProxy<'a>,
    is_dark: bool,
) -> anyhow::Result<()> {
    // Connect to the settings daemon
    // Retrieve default theme and apply colors to it
    let (builder_config, default) = if is_dark {
        (
            cosmic_theme::ThemeBuilder::dark_config()?,
            cosmic_theme::Theme::dark_default(),
        )
    } else {
        (
            cosmic_theme::ThemeBuilder::light_config()?,
            cosmic_theme::Theme::light_default(),
        )
    };

    let mut theme = match cosmic_theme::ThemeBuilder::get_entry(&builder_config) {
        Ok(entry) => entry,
        Err((errs, entry)) => {
            for err in errs {
                log::error!("failed to get theme: {}", err);
            }
            entry
        }
    };
    let scheme = &material_theme.schemes.dark;

    theme.write_entry(&builder_config)?;
    theme = theme.accent(*argb_to_srgba(scheme.primary));
    theme = theme.bg_color((*argb_to_srgba(scheme.background)).into());
    theme = theme.text_tint(*argb_to_srgba(scheme.on_background));
    theme = theme.neutral_tint(*argb_to_srgba(scheme.secondary_container));
    let theme = theme.build();
    let theme_config = if theme.is_dark {
        cosmic_theme::Theme::dark_config()
    } else {
        cosmic_theme::Theme::light_config()
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
