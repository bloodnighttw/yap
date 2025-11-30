use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config;

lazy_static::lazy_static! {
    pub static ref LOG_ENV: String = format!("{}_LOG_LEVEL", config::PROJECT_NAME.clone());
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

pub fn init() -> color_eyre::Result<()> {
    let directory = config::get_data_dir();
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(LOG_FILE.clone());
    let log_file = std::fs::File::create(log_path.clone())?;
    
    // Try RUST_LOG first, then YAP_LOG_LEVEL, fall back to INFO
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_from_env(LOG_ENV.clone()))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    let file_subscriber = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(env_filter);
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .try_init()?;
    
    // Log where the file is being written
    tracing::info!("Logging to file: {:?}", log_path);
    Ok(())
}
