use std::sync::Arc;
use tokio::sync::RwLock;
use http::StatusCode;

use crate::error::{Result, SoundbaseError};
use crate::model::song_like::SourceMetadataDetermination;
use std::collections::HashMap;
use crate::model::spotify;

mod song_like_handler;
mod album_of_week;
mod top20_of_week;

type DB = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
type Dissects = Arc<Vec<SourceMetadataDetermination>>;
type Spotify = Arc<RwLock<super::model::spotify::Spotify>>;

pub async fn heartbeat() -> Result<impl warp::Reply, std::convert::Infallible> {
    println!("Received a Heartbeat request.");
    println!();
    Ok(reply(String::from(""), StatusCode::OK))
}

pub async fn song_fav(body: bytes::Bytes, spotify: Spotify, dissects: Dissects) -> Result<impl warp::Reply, std::convert::Infallible> {
    let resp = song_like_handler::consume_like_message(spotify, &dissects, body.to_vec()).await;

    match resp {
        Ok(r) => Ok(reply(r, http::StatusCode::OK)),
        Err(e) => {
            println!("\tResponding with Error => {:?}", e.msg);
            Ok(reply(e.msg.as_bytes().to_vec(), e.http_code))
        }
    }
}

pub async fn fetch_top_20_of_week(db: DB) -> Result<impl warp::Reply, std::convert::Infallible> {
    match db.get() {
        Ok(mut db_conn) => {
            let response = top20_of_week::fetch_new_rockantenne_top20_of_week(&mut db_conn);
            match response {
                Ok(..) => Ok(reply(String::from(""), http::StatusCode::OK)),
                Err(e) => {
                    println!("\tResponding with Error => {}", e.msg);
                    Ok(reply(e.msg, e.http_code))
                }
            }
        }
        Err(e) => {
            Ok(reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

pub async fn fetch_album_of_week(db: DB) -> Result<impl warp::Reply, std::convert::Infallible> {
    match db.get() {
        Ok(mut db_conn) => {
            let response = album_of_week::fetch_new_rockantenne_album_of_week(&mut db_conn);
            match response {
                Ok(..) => Ok(reply(String::from(""), StatusCode::OK)),
                Err(e) => {
                    println!("\tResponding with Error => {:?}", e.msg);
                    Ok(reply(e.msg, e.http_code))
                }
            }
        }
        Err(e) => {
            Ok(reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

pub async fn spotify_start_auth(wrapper: Spotify) -> Result<impl warp::Reply, std::convert::Infallible> {
    let spotify = wrapper.read().await;
    let uri = spotify.get_authorization_url().await.to_string();
    Ok(reply(uri, http::StatusCode::OK))
}

pub async fn spotify_auth_callback(wrapper: Spotify, query: String) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut spotify = wrapper.write().await;
    match spotify.finish_initialization_with_code(query.as_str()).await {
        Ok(_) => {
            Ok(reply("Successful Authentication.".to_owned(), StatusCode::OK))
        }
        Err(e) => {
            println!("\tResponding with Error => {:?}", e);
            Ok(reply(e.msg, e.http_code))
        }
    }
}


fn reply<T: warp::Reply>(r: T, status: StatusCode) -> impl warp::Reply {
    warp::reply::with_status(r, status)
}

fn get_selector(selector: &'static str) -> Result<scraper::Selector> {
    let sel = scraper::Selector::parse(selector);
    match sel {
        Ok(s) => Ok(s),
        Err(e) => {
            Err(SoundbaseError {
                http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!("{:?}", e),
            })
        }
    }
}
