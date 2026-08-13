use nenechi_cli::jellyfin::series::organize;
use crate::jellyfin::{jellyfin_anime_config, setup};
use nenechi_cli::jellyfin::anime::AnimeResolver;
use nenechi_cli::jellyfin::series_metadata::SeriesMetadataRepository;
use crate::config::cli_config;
use crate::jellyfin::ANIME_DIRECTORIES;

#[test]
fn test_organise() {
    setup();

    let expected = ANIME_DIRECTORIES.len() * 12;

    let jellyfin_config = jellyfin_anime_config();
    let cli_config = cli_config();
    let target = jellyfin_config.targets[0].clone();
    let resolver = AnimeResolver::from_config(&cli_config).unwrap();
    let metadata_repository = SeriesMetadataRepository::from_config(&cli_config);

    let result = organize(&target, &resolver, &metadata_repository);
    println!("organize: {:?}", result);

    assert!(result.is_ok());
    assert_eq!(expected, result.unwrap())
}
