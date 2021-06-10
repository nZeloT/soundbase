use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use rspotify::oauth2::{SpotifyOAuth, SpotifyClientCredentials};
use http::StatusCode;

use crate::error::{Result, SoundbaseError};
use crate::model::song_like::SourceMetadataDissect;
use crate::model::spotify::Spotify;

mod analytics_handler;
mod song_like_handler;
mod album_of_week;
mod top20_of_week;
mod spotify_handler;

type DB = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
type Dissects = Arc<Vec<SourceMetadataDissect>>;

pub async fn heartbeat() -> Result<impl warp::Reply, std::convert::Infallible> {
    println!("Received a Heartbeat request for analytics.");
    println!();
    Ok(reply(String::from(""), StatusCode::OK))
}

pub async fn analytics_message(body: bytes::Bytes, db: DB) -> Result<impl warp::Reply, std::convert::Infallible> {
    match db.get() {
        Ok(mut db_conn) => {
            let _ = analytics_handler::consume_analytics_message(&mut db_conn, body.to_vec());
            Ok(reply(String::from(""), http::StatusCode::ACCEPTED))
        }
        Err(e) => {
            Ok(reply(e.to_string(), http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

pub async fn song_fav(body: bytes::Bytes, db: DB, dissects: Dissects) -> Result<impl warp::Reply, std::convert::Infallible> {
    match db.get() {
        Ok(mut db_conn) => {
            let resp = song_like_handler::consume_like_message(&mut db_conn, &dissects, body.to_vec());

            match resp {
                Ok(r) => Ok(reply(r, http::StatusCode::OK)),
                Err(e) => {
                    println!("\tResponding with Error => {:?}", e.msg);
                    Ok(reply(e.msg.as_bytes().to_vec(), e.http_code))
                }
            }
        }
        Err(e) => {
            Ok(reply(e.to_string().as_bytes().to_vec(), http::StatusCode::INTERNAL_SERVER_ERROR))
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

pub async fn spotify_start_auth(wrapper: Arc<RwLock<Spotify>>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let spotify = wrapper.read().await;
    let uri = spotify.request_authorization_token().await.clone();
    Ok(reply(uri, http::StatusCode::OK))
}

pub async fn spotify_auth_callback(mut wrapper: Arc<RwLock<Spotify>>, query: HashMap<String, String>) -> Result<impl warp::Reply, std::convert::Infallible> {
    match query.get("code") {
        Some(code) => {
            let mut spotify = wrapper.write().await;
            match spotify.finish_initialization_with_code(code.as_str()).await {
                Ok(_) => {
                    Ok(reply("Successful Authentication.".to_owned(), StatusCode::OK))
                }
                Err(e) => {
                    println!("\tResponding with Error => {:?}", e);
                    Ok(reply(e.msg, e.http_code))
                }
            }
        }
        None => {
            println!("\tResponding with Error => No 'code' given as query parameter!");
            Ok(reply("Couldn't find 'code' as query parameter!".to_owned(), StatusCode::BAD_REQUEST))
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
