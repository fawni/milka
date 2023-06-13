use miette::{Context, IntoDiagnostic};
use owo_colors::OwoColorize;
use tokio::io::AsyncWriteExt;

use crate::api::{FavoritesResponse, VideoResponse};

mod api;
mod db;

#[tokio::main]
async fn main() -> miette::Result<()> {
    dotenvy::dotenv().into_diagnostic()?;
    let database = db::open().await?;

    twink::info!("Fetching favorites...");
    let client = reqwest::Client::new();
    let sec_uid = std::env::var("SEC_UID")
        .into_diagnostic()
        .wrap_err("SEC_UID")?;
    let cookie = std::env::var("COOKIE")
        .into_diagnostic()
        .wrap_err("COOKIE")?;

    let mut cursor = String::from("0");
    let mut page_counter = 1_u32;

    fs_err::tokio::create_dir_all("output")
        .await
        .into_diagnostic()?;

    'outer: loop {
        let url = format!("https://www.tiktok.com/api/favorite/item_list/?aid=1988&count=30&cursor={cursor}&secUid={sec_uid}");
        let res = client
            .get(url)
            .header("cookie", &cookie)
            .send()
            .await
            .into_diagnostic()?
            .json::<FavoritesResponse>()
            .await;

        let res = match res {
            Ok(res) => res,
            Err(e) => {
                twink::err!("{e:?}");
                continue;
            }
        };

        for vid in res.favorites {
            let id = vid.id;
            if database.get_status(&id).await.is_ok() {
                break 'outer;
            }
            database.set(&id, 0).await?;
        }
        twink::info!("Fetched favorites page: {page_counter}",);
        if !res.has_more {
            break;
        }
        cursor = res.next_cursor;
        page_counter += 1;
    }

    let ids = database.get_new_favorites().await?;
    if ids.is_empty() {
        twink::warn!("No new favorites found! Exiting...");
        return Ok(());
    }
    twink::info!("Found {} favorites!", ids.len().bold());
    twink::info!("Starting downloads...");

    for (i, id) in ids.iter().enumerate() {
        if database.get_status(id).await? == 1 {
            continue;
        };

        let url = format!("https://api2.musical.ly/aweme/v1/feed/?aweme_id={id}");
        let res = match client.get(url).send().await {
            Ok(res) => {
                let Ok(res) = res.json::<VideoResponse>().await else {
                    twink::err!("Error      {} ({})", id.bold(), "???".red());
                    continue;
                };

                res
            }
            Err(e) => {
                twink::err!("{id}: {e:?}");
                continue;
            }
        };

        let aweme = &res.aweme_list[0];

        let Some(vid_url) = aweme.video.play_addr.url_list.get(0) else {
            twink::err!("Error      {} ({})", id.bold(), "deleted".red());
            database.set(id, 8).await?;
            continue;
        };

        if vid_url.ends_with(".mp3") {
            twink::warn!("Skipped    {} ({})", id.bold(), "slideshow".yellow());
            database.set(id, 2).await?;
            continue;
        }
        let author = &aweme.author.username;

        let res = match client.get(vid_url).send().await {
            Ok(res) => {
                let Ok(res) = res.bytes().await else {
                    twink::err!("Error      {} ({})", id.bold(), "???".red());
                    continue;
                };

                res
            }
            Err(e) => {
                twink::err!("{id}: {e:?}");
                continue;
            }
        };

        let mut file = fs_err::tokio::File::create(format!("output/{id} - {author}.mp4"))
            .await
            .into_diagnostic()?;
        file.write_all(&res).await.into_diagnostic()?;

        twink::info!("Downloaded {} ({}/{})", id.bold(), i + 1, ids.len());
        database.set(id, 1).await?;
    }

    Ok(twink::info!("Done!"))
}
