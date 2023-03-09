use owo_colors::OwoColorize;
use tokio::{fs, io::AsyncWriteExt};

use crate::api::FavoritesResponse;
use crate::api::VideoResponse;

mod api;
mod db;
mod log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    fs::create_dir_all("output").await?;
    let database = db::open().await?;

    info!("Fetching favorites...");
    let client = reqwest::Client::new();
    let sec_uid = std::env::var("SEC_UID")?;
    let cookie = std::env::var("COOKIE")?;
    let mut cursor = "0".to_owned();
    let mut page_counter = 1_u32;

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

        for vid in res.favorites {
            let id = vid.id;
            if database.get_status(&id).await.is_ok() {
                break 'outer;
            }
            database.set(id, 0).await?;
        }
        info!("Fetched favorites page: {page_counter}",);
        if !res.has_more {
            break;
        }
        cursor = res.next_cursor;
        page_counter += 1;
    }

    let ids = database.get_new_favorites().await?;
    if ids.is_empty() {
        warn!("No new favorites found! Exiting...");
        return Ok(());
    }
    info!("Found {} favorites!", ids.len().bold());
    info!("Starting downloads...");

    for (i, id) in ids.clone().into_iter().enumerate() {
        if database.get_status(&id).await? == 1 {
            continue;
        };

        let url = format!("https://api2.musical.ly/aweme/v1/feed/?aweme_id={id}");
        let res = match client.get(url).send().await {
            Ok(res) => {
                let Ok(res) = res.json::<VideoResponse>().await else {
                    err!("Error      {} ({})", id.bold(), "???".red());
                    continue;
                };

                res
            }
            Err(e) => {
                err!("{id}: {e:?}");
                continue;
            }
        };

        let aweme = res.aweme_list[0].clone();

        let Some(vid_url) = aweme.video.play_addr.url_list.get(0) else {
            err!("Error      {} ({})", id.bold(), "deleted".red());
            database.set(id, 8).await?;
            continue;
        };

        if vid_url.ends_with(".mp3") {
            warn!("Skipped    {} ({})", id.bold(), "slideshow".yellow());
            database.set(id, 2).await?;
            continue;
        }
        let author = aweme.author.username;

        let res = match client.get(vid_url).send().await {
            Ok(res) => {
                let Ok(res) = res.bytes().await else {
                    err!("Error      {} ({})", id.bold(), "???".red());
                    continue;
                };

                res
            }
            Err(e) => {
                err!("{id}: {e:?}");
                continue;
            }
        };

        let mut file = fs::File::create(format!("output/{id} - {author}.mp4")).await?;
        file.write_all(&res).await?;

        info!("Downloaded {} ({}/{})", id.bold(), i + 1, ids.len());
        database.set(id, 1).await?;
    }

    Ok(info!("Done!"))
}
