use std::path::Path;

use anyhow::Context;
use cosmic_config::CosmicConfigEntry;
use material_colors::{
    color::Argb,
    image::{FilterType, ImageReader},
    theme::{Theme, ThemeBuilder},
};
use palette::Srgba;

pub mod config;
pub mod options;

/// Generate colors using material-colors
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
    is_dark: bool,
) -> anyhow::Result<()> {
    // Retrieve default theme and apply colors to it
    let (builder_config, _default) = if is_dark {
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
