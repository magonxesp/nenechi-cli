mod anime;
mod client;

pub use anime::{
    AlternativeTitles, Anime, AnimeDetails, AnimeSearchEntry, AnimeSearchResponse, Broadcast,
    Genre, MainPicture, MyListStatus, Paging, Recommendation, RelatedAnime, RelatedManga,
    RelationType, StartSeason, Statistics, StatusStatistics, Studio,
};
pub use client::{Client, ClientError};
