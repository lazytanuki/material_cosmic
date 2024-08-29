use std::path::Path;

use anyhow::Context;
use wallust::colors::Colors;

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
pub fn apply_colors(
    colors: &Colors,
    cache_path: &Path,
    skip_sequences: bool,
) -> anyhow::Result<()> {
    if !skip_sequences {
        colors.sequences(cache_path, None)?
    }

    Ok(())
}
