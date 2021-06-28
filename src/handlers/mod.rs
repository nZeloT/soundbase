use std::sync::Arc;
use tokio::sync::RwLock;
use http::StatusCode;

use crate::error::{Result, SoundbaseError};
use crate::model::song_like::SourceMetadataDetermination;
use std::collections::HashMap;
use crate::handlers::spotify_handler::get_token_from_refresh_token;

mod analytics_handler;
mod song_like_handler;
mod album_of_week;
mod top20_of_week;
mod spotify_handler;

type DB = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
type Dissects = Arc<Vec<SourceMetadataDetermination>>;
type Spotify = Arc<RwLock<super::model::spotify::Spotify>>;

pub async fn heartbeat() -> Result<impl warp::Reply, std::convert::Infallible> {
    println!("Received a Heartbeat request.");
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
    let full_url = "http://dummy.adress/?".to_owned() + query.as_str();
    let r = |reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*");


    let url = match url::Url::parse(&full_url) {
        Ok(url) => url,
        Err(e) => {
            return Ok(reply(r(warp::reply::with_header(e.to_string(), "content-type", "text/plain")), StatusCode::INTERNAL_SERVER_ERROR))
        }
    };
    let params : HashMap<_, _> = url.query_pairs().collect();

    // 1. $GET['code'] is set
    if params.contains_key("code") || params.contains_key("error") {
        let json_response = spotify_handler::get_token_from_code(wrapper, query).await;
        Ok(reply(r(warp::reply::with_header(format!("
        <script type=\"text/javascript\">
            window.opener.postMessage( {}, \"*\" );
            window.close();
        </script>
        ", json_response), "content-type", "text/html")), StatusCode::OK))

    } else if params.get("action").map_or(false, |action| action == "refresh") {
        match get_token_from_refresh_token(wrapper).await {
            Ok(response) => Ok(reply(r(warp::reply::with_header(response, "content-type", "application/json")), StatusCode::OK)),
            Err(e) => Ok(reply(r(warp::reply::with_header(e.msg, "content-type", "text/plain")), StatusCode::UNAUTHORIZED))
        }
    } else {
        //redirect to authorization
        let spot = wrapper.read().await;
        Ok(reply(r(warp::reply::with_header("".to_string(), "Location", spot.get_authorization_url().await)), StatusCode::PERMANENT_REDIRECT))
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
