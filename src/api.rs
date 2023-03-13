use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FavoritesResponse {
    #[serde(rename = "cursor")]
    pub next_cursor: String,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "itemList")]
    pub favorites: Vec<Favorite>,
}

#[derive(Deserialize, Debug)]
pub struct Favorite {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct VideoAuthor {
    #[serde(rename = "unique_id")]
    pub username: String,
}

#[derive(Deserialize, Debug)]
pub struct VideoResponse {
    pub aweme_list: Vec<Aweme>,
}

#[derive(Deserialize, Debug)]
pub struct Aweme {
    pub author: VideoAuthor,
    pub video: Video,
}

#[derive(Deserialize, Debug)]
pub struct Video {
    pub play_addr: PlayAddr,
}

#[derive(Deserialize, Debug)]
pub struct PlayAddr {
    pub url_list: Vec<String>,
}
