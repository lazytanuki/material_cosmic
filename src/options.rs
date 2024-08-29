//! Options for both this lib and wallust.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Options for both this lib and wallust
#[derive(Parser)]
pub struct Options {
    #[clap(subcommand)]
    pub mode: Cmd,
    /// Path used to cache colors, wallpaper settings, and templates. If not set, cache files will be stored at `~/.cache/cosmic-wallust`.
    #[clap(short, long)]
    pub cache_dir: Option<PathBuf>,
    /// Overwrite cache, if any
    #[clap(long)]
    pub overwrite_cache: bool,
    /// Do not use per-wallpaper cached settings
    #[clap(long)]
    pub skip_cached_settings: bool,
    /// Do not generate templates
    #[clap(long)]
    pub skip_template_generation: bool,
    /// Wallust backend to use
    #[clap(short, long)]
    pub backend: Option<wallust::backends::Backend>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Generate colors for a single wallpaper
    #[clap(alias = "gen")]
    Generate {
        /// Path of the wallpaper for which to generate colors
        #[clap(short, long)]
        wallpaper: PathBuf,
    },
    /// Run in a loop to monitor wallpaper changes
    Daemon {},
}
