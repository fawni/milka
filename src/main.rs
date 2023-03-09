use owo_colors::OwoColorize;
use tokio::{fs, io::AsyncWriteExt};

use crate::api::ApiResponse;
use crate::api::FavoritesResponse;

mod api;
mod db;
mod log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    fs::create_dir_all("output").await?;
    let database = db::open().await?;

    info!("Fetching favorites...");
    let mut cursor = "0".to_owned();
    let mut counter = 1_u32;
    let client = reqwest::Client::new();
    let sec_uid = std::env::var("SEC_UID")?;
    let cookie = std::env::var("COOKIE")?;

    'outer: loop {
        let url = format!("https://www.tiktok.com/api/favorite/item_list/?aid=1988&count=30&cursor={cursor}&secUid={sec_uid}");
        let res = client
            .get(url)
            .header("cookie", &cookie)
            .send()
            .await?
            .json::<FavoritesResponse>()
            .await;
        let res = match res {
            Ok(res) => res,
            Err(e) => {
                err!("{e:?}");
                continue;
            }
        };

        for tt in res.favorites {
            let id = tt.id;
            if database.get(&id).await.is_some() {
                break 'outer;
            }
            database.set(&id, 0).await?;
        }
        info!("Fetched favorites page: {counter}",);
        if !res.has_more {
            break;
        }
        cursor = res.next_cursor;
        counter += counter;
    }

    let ids = database.get_favorites().await?.unwrap();
    if ids.is_empty() {
        warn!("No new favorites found! Exiting...");
        return Ok(());
    }
    info!("Found {} favorites!", ids.len().bold());
    info!("Starting downloads...");

    for (i, id) in ids.clone().into_iter().enumerate() {
        let url = format!("https://api2.musical.ly/aweme/v1/feed/?aweme_id={id}");
        let res = match client.get(url).send().await {
            Ok(res) => res.json::<ApiResponse>().await?,
            Err(e) => {
                err!("{id}: {e:?}");
                continue;
            }
        };

        let aweme = res.aweme_list[0].clone();
        let vid_url = match aweme.video.play_addr.url_list.get(0) {
            Some(url) => url,
            None => {
                database.set(&id.to_owned(), 8).await?;
                err!("Error      {} ({})", id.bold(), "deleted".red());
                continue;
            }
        };
        if vid_url.ends_with(".mp3") {
            database.set(&id.to_owned(), 2).await?;
            warn!("Skipped    {} ({})", id.bold(), "slideshow".yellow());
            continue;
        }
        let author = aweme.author.username;

        if database.get(&id).await == Some(1) {
            continue;
        };

        let res = match client.get(vid_url).send().await {
            Ok(res) => res.bytes().await?,
            Err(e) => {
                err!("{id}: {e:?}");
                continue;
            }
        };

        let mut file = fs::File::create(format!("output/{id} - {author}.mp4")).await?;
        file.write_all(&res).await?;

        database.set(&id.to_owned(), 1).await?;
        info!("Downloaded {} ({}/{})", id.bold(), i + 1, ids.len());
    }

    Ok(info!("Done!"))
}
