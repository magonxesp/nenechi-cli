use reqwest::blocking::RequestBuilder;
use serde::Deserialize;

use crate::{Client, ClientError};

const ANIME_DETAILS_FIELDS: &str = "id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,created_at,updated_at,media_type,status,genres,my_list_status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,pictures,background,related_anime,related_manga,recommendations,studios,statistics";

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AnimeSearchResponse {
    pub data: Vec<AnimeSearchEntry>,
    pub paging: Paging,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AnimeSearchEntry {
    pub node: Anime,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Anime {
    pub id: u64,
    pub title: String,
    pub main_picture: Option<MainPicture>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AnimeDetails {
    pub id: u64,
    pub title: String,
    pub main_picture: Option<MainPicture>,
    pub alternative_titles: AlternativeTitles,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub synopsis: Option<String>,
    pub mean: Option<f64>,
    pub rank: Option<u64>,
    pub popularity: Option<u64>,
    pub num_list_users: u64,
    pub num_scoring_users: u64,
    pub nsfw: String,
    pub created_at: String,
    pub updated_at: String,
    pub media_type: String,
    pub status: String,
    pub genres: Vec<Genre>,
    pub my_list_status: Option<MyListStatus>,
    pub num_episodes: u64,
    pub start_season: Option<StartSeason>,
    pub broadcast: Option<Broadcast>,
    pub source: Option<String>,
    pub average_episode_duration: Option<u64>,
    pub rating: Option<String>,
    pub pictures: Vec<MainPicture>,
    pub background: Option<String>,
    pub related_anime: Vec<RelatedAnime>,
    pub related_manga: Vec<RelatedManga>,
    pub recommendations: Vec<Recommendation>,
    pub studios: Vec<Studio>,
    pub statistics: Statistics,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct MainPicture {
    pub medium: String,
    pub large: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AlternativeTitles {
    pub synonyms: Vec<String>,
    pub en: Option<String>,
    pub ja: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Genre {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct MyListStatus {
    pub status: String,
    pub score: u8,
    pub num_episodes_watched: u64,
    pub is_rewatching: bool,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub priority: u8,
    pub num_times_rewatched: u64,
    pub rewatch_value: u8,
    pub tags: Vec<String>,
    pub comments: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct StartSeason {
    pub year: u64,
    pub season: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Broadcast {
    pub day_of_the_week: String,
    pub start_time: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RelatedAnime {
    pub node: Anime,
    pub relation_type: RelationType,
    pub relation_type_formatted: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RelatedManga {
    pub node: Anime,
    pub relation_type: RelationType,
    pub relation_type_formatted: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Sequel,
    Prequel,
    AlternativeSetting,
    AlternativeVersion,
    SideStory,
    ParentStory,
    Summary,
    FullStory,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Recommendation {
    pub node: Anime,
    pub num_recommendations: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Studio {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Statistics {
    pub status: StatusStatistics,
    pub num_list_users: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct StatusStatistics {
    pub watching: String,
    pub completed: String,
    pub on_hold: String,
    pub dropped: String,
    pub plan_to_watch: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Paging {
    pub previous: Option<String>,
    pub next: Option<String>,
}

impl Client {
    pub fn get_anime(&self, anime_id: u64) -> Result<AnimeDetails, ClientError> {
        let response = self
            .get(&format!("anime/{anime_id}"))
            .query(&[("fields", ANIME_DETAILS_FIELDS)])
            .send()
            .map_err(ClientError::Http)?
            .error_for_status()
            .map_err(ClientError::Http)?;

        response.json().map_err(ClientError::Http)
    }

    pub fn search_anime(&self, name: &str) -> Result<AnimeSearchResponse, ClientError> {
        let response = self
            .search_anime_request(name)?
            .send()
            .map_err(ClientError::Http)?
            .error_for_status()
            .map_err(ClientError::Http)?;

        response.json().map_err(ClientError::Http)
    }

    fn search_anime_request(&self, name: &str) -> Result<RequestBuilder, ClientError> {
        if name.trim().is_empty() {
            return Err(ClientError::EmptySearchQuery);
        }

        Ok(self.get("anime").query(&[("q", name)]))
    }
}
