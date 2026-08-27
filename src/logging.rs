use crate::config::LoggingConfig;
use crate::fs::expand_user_dir;
use std::fs::{self, File};
use std::io;
use std::path::Path;

fn open_log_file(path: impl AsRef<Path>) -> io::Result<File> {
    let path = path.as_ref();

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    fern::log_file(path)
}

/// Configures the application logger with console and file outputs.
pub fn configure(config: &LoggingConfig) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(config.level.as_level_filter())
        .chain(std::io::stdout())
        .chain(open_log_file(expand_user_dir(&config.file))?)
        .apply()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn creates_the_log_directory_recursively_before_opening_the_file() {
        let temporary = tempdir().unwrap();
        let log_file = temporary.path().join("state/nenechi/logs/nenechi.log");

        open_log_file(&log_file).unwrap();

        assert!(log_file.is_file());
    }
}
