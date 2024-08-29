use anyhow::bail;
use clap::Parser;
use cosmic_wallust::{generate_colors, options};
use directories::ProjectDirs;
use log::LevelFilter;

fn main() -> anyhow::Result<()> {
    // Handle arguments
    let args = options::Options::parse();
    let cache_dir = match args.cache_dir {
        Some(c) => c,
        None => {
            let Some(project_dirs) = ProjectDirs::from("", "", env!("CARGO_CRATE_NAME")) else {
                bail!("no valid home directory path could be retrieved from the operating system")
            };
            project_dirs.cache_dir().to_owned()
        }
    };

    // Init logger
    simplelog::TermLogger::init(
        LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("unable to init logger");

    match args.mode {
        options::Cmd::Generate { wallpaper } => {
            let wallust_config = wallust::config::Config {
                backend_user: args.backend,
                backend: args.backend.unwrap_or_default(),
                true_th: 17,
                ..Default::default()
            };
            let colors = generate_colors(
                wallpaper.as_path(),
                &wallust_config,
                &cache_dir,
                args.overwrite_cache,
            )?;
            colors.print();
        }
        options::Cmd::Daemon {} => todo!(),
    }

    Ok(())
}
