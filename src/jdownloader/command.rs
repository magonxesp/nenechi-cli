use crate::config::CliConfig;
use clap::Subcommand;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use crate::jdownloader::{JDownloader, JobId};

#[derive(Clone, Debug, Subcommand)]
pub enum JDownloaderCommands {
    Download {
        urls: Vec<String>,
        #[arg(short, long)]
        package_name: Option<String>,
        #[arg(short, long)]
        destination: Option<String>,
    },
    Progress {
        job_id: JobId,
    }
}

impl Display for JDownloaderCommands {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Download { .. } => formatter.write_str("download"),
            Self::Progress { .. } => formatter.write_str("progress"),
        }
    }
}

pub fn execute_jdownloader_command(command: &JDownloaderCommands) -> Result<(), String> {
    let client = JDownloader::get_instance();
    let result = match command {
        JDownloaderCommands::Download {
            urls,
            package_name,
            destination
        } => download(
            client,
            &urls,
            package_name.clone(),
            destination.clone().map(PathBuf::from)
        ),
        JDownloaderCommands::Progress { job_id } => check_progress(&client, *job_id)
    };

    result
        .map(|_| ())
        .map_err(|error| format!("subcommand {} failed: {error}", command))
}

fn download(
    client: &JDownloader,
    urls: &Vec<String>,
    package_name: Option<String>,
    destination: Option<PathBuf>
) -> Result<(), String> {
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("error getting current working directory: {}", error))?
        .to_path_buf();

    let job_id = client.download(
        &urls,
        &destination.unwrap_or(current_dir),
        &package_name.unwrap_or("".to_string())
    ).map_err(|error| format!("download failed: {}", error))?;

    println!("{}", job_id);
    Ok(())
}

fn check_progress(client: &JDownloader, job_id: JobId) -> Result<(), String> {
    let progress = client.check_progress(job_id)
        .map_err(|error| format!("check progress failed: {error}"))?;

    let json = serde_json::to_string_pretty(&progress)
        .map_err(|error| format!("write progress failed: {error}"))?;

    println!("{}", json);
    Ok(())
}
