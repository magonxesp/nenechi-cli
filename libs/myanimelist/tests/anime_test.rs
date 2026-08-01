use std::env;

use nenechi_myanimelist::{Client, MediaType};

fn load_env() {
    dotenvy::dotenv().expect("no se pudo cargar el fichero .env");
}

#[test]
fn it_should_search_by_anime_title() {
    load_env();

    let api_key = env::var("MYANIMELIST_API_KEY").expect("MYANIMELIST_API_KEY debe estar definida");
    let client = Client::new(api_key).unwrap();

    let response = client.search_anime("New Game").unwrap();

    assert!(
        response
            .data
            .iter()
            .any(|result| result.node.title == "New Game!")
    );
}

#[test]
fn it_should_search_a_long_title_without_losing_the_season_suffix() {
    load_env();

    let api_key = env::var("MYANIMELIST_API_KEY").expect("MYANIMELIST_API_KEY debe estar definida");
    let client = Client::new(api_key).unwrap();

    let response = client
        .search_anime(
            "Honzuki no Gekokujou Shisho ni Naru Tame ni wa Shudan wo Erandeiraremasen 3rd Season",
        )
        .unwrap();

    assert!(response.data.iter().any(|result| result.node.id == 42429));
}

#[test]
fn it_should_get_anime_by_id() {
    load_env();

    let api_key = env::var("MYANIMELIST_API_KEY").expect("MYANIMELIST_API_KEY debe estar definida");
    let client = Client::new(api_key).unwrap();

    let anime = client.get_anime(31953).unwrap();
    assert_eq!(true, anime.is_some(), "anime not found");

    let anime = anime.unwrap();
    assert_eq!(anime.id, 31953);
    assert_eq!(anime.title, "New Game!");
    assert_eq!(anime.media_type, MediaType::Tv);
    assert_eq!(anime.num_episodes, 12);
}

#[test]
fn it_should_deserialize_known_media_types() {
    let cases = [
        ("unknown", MediaType::Unknown),
        ("tv", MediaType::Tv),
        ("ova", MediaType::Ova),
        ("movie", MediaType::Movie),
        ("special", MediaType::Special),
        ("ona", MediaType::Ona),
        ("music", MediaType::Music),
    ];

    for (value, expected) in cases {
        let media_type = serde_json::from_str::<MediaType>(&format!("\"{value}\""))
            .expect("media type should deserialize");
        assert_eq!(media_type, expected);
    }
}

#[test]
fn it_should_fallback_to_unknown_for_unrecognized_media_types() {
    let media_type = serde_json::from_str::<MediaType>("\"unexpected\"")
        .expect("unknown media type should deserialize");

    assert_eq!(media_type, MediaType::Unknown);
}
