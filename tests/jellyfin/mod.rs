use std::path::Path;
use std::sync::OnceLock;
use std::fs;
use nenechi_cli::jellyfin::config::{JellyfinConfig, JellyfinConfigRoot};
use crate::config;

mod series;

pub fn setup() {
    config::setup();

    fs::remove_dir_all("tmp").unwrap();
    create_mock_anime_directories();
}

pub fn jellyfin_anime_config() -> JellyfinConfig {
    let config: JellyfinConfigRoot = serde_yaml::from_str(include_str!("../fixtures/config/jellyfin.anime.yaml")).unwrap();
    config.jellyfin
}

pub const ANIME_DIRECTORIES: &'static [&'static str] = &[
    "Honzuki no Gekokujou Shisho ni Naru Tame ni wa Shudan wo Erandeiraremasen",
    "Honzuki no Gekokujou Shisho ni Naru Tame ni wa Shudan wo Erandeiraremasen - Ryoushu no Youjo",
    "Honzuki no Gekokujou Shisho ni Naru Tame ni wa Shudan wo Erandeiraremasen 2nd Season",
    "Honzuki no Gekokujou Shisho ni Naru Tame ni wa Shudan wo Erandeiraremasen 3rd Season",
    "Kobayashi-san Chi no Maid Dragon",
    "Kobayashi-san Chi no Maid Dragon S",
    "Kobayashi-san Chi no Maid Dragon Specials",
    "Uma Musume Cinderella Gray",
    "Uma Musume Pretty Derby",
    "Uma Musume Pretty Derby - BNW no Chikai",
    "Uma Musume Pretty Derby - Road to the Top",
    "Uma Musume Pretty Derby Season 2",
    "Uma Musume Pretty Derby Season 3",
];

static ANIME_DIRECTORIES_CREATED: OnceLock<()> = OnceLock::new();

fn create_mock_anime_directories() {
    if ANIME_DIRECTORIES_CREATED.get().is_some() {
        return;
    }

    let base_directory = Path::new("tmp/anime/source");

    for directory in ANIME_DIRECTORIES {
        let anime_directory = base_directory.join(directory);
        fs::create_dir_all(&anime_directory).unwrap();

        for episode in 0..12 {
            let episode_file = anime_directory.join(format!("{}.mp4", episode));
            fs::File::create(episode_file).unwrap();
        }
    }

    let _ = ANIME_DIRECTORIES_CREATED.set(());
}
