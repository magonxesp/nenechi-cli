use crate::config::LoggingConfig;
use crate::fs::expand_user_dir;

/// Configures the application logger with console and file outputs.
pub fn configure(config: &LoggingConfig) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(config.level.as_level_filter())
        .chain(std::io::stdout())
        .chain(fern::log_file(expand_user_dir(&config.file))?)
        .apply()?;

    Ok(())
}
