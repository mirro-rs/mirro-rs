mod cli;
mod config;
mod dbg;
mod direct;
#[cfg(test)]
mod test;
mod tui;
mod utils;

use std::sync::{Arc, Mutex};

use tracing::error;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
use self::config::watch_config;

#[tokio::main]
async fn main() {
    let args = <cli::ArgConfig as clap::Parser>::parse();

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let (config, file) = config::read_config_file(args.general.config.as_ref());

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    if !check_outfile(&args.general) && !check_outfile(&config.general) {
        exit("outfile");
    }

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    if !check_outfile(&args.general) {
        exit("outfile");
    }

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let config = config::Configuration::from((args, config));

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    let config = config::Configuration::from(args);

    dbg::log(config.direct);

    if config.direct {
        if let Err(e) = direct::begin(config).await {
            error!("{e}")
        }
    } else {
        let config = Arc::new(Mutex::new(config));

        #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
        watch_config(file, Arc::clone(&config));

        let _ = tui::start(config).await;
    }
    std::process::exit(0);
}

pub fn exit(value: &str) -> ! {
    let cmd = clap::Command::new("mirro-rs");
    let mut err = clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(&cmd);
    err.insert(
        clap::error::ContextKind::InvalidArg,
        clap::error::ContextValue::String(format!("--{value}")),
    );

    err.insert(
        clap::error::ContextKind::InvalidValue,
        clap::error::ContextValue::String(String::default()),
    );
    err.exit();
}

fn check_outfile(config: &cli::Args) -> bool {
    if let Some(ref outfile) = config.outfile {
        if outfile.to_string_lossy().ends_with('/') || outfile.to_string_lossy().is_empty() {
            exit("outfile");
        }
        true
    } else {
        false
    }
}
