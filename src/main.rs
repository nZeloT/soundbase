use crate::song_db::SongDB;
use std::net::SocketAddr;
use std::sync::Arc;

mod error;
mod db;
mod song_db;
mod album_of_week;
mod top20_of_week;
mod analytics;
mod analytics_handler;
mod analytics_protocol_generated;
mod song_like;
mod song_like_handler;
mod song_like_protocol_generated;

#[derive(Clone)]
struct RequestPayload {
    db_pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    dissects: std::sync::Arc<song_like::SourceMetadataDissectConfig>,
}

#[tokio::main]
async fn main() {
    let db = db::setup_db().expect("Failed to create DB!");

    let metadata_dissect = song_like::SourceMetadataDissectConfig::load_from_file("./config.json");
    println!("Read the following Metadata dissects:");
    println!("\t{:?}", metadata_dissect);
    println!();

    let api = filters::endpoints(db, Arc::new(metadata_dissect.sources));


    let env_ip_str = match std::env::var("SERVER_IP") {
        Ok(given_ip) => given_ip,
        Err(_) => "192.168.2.111:3333".to_string()
    };
    let sock_addr: SocketAddr = env_ip_str.parse().unwrap();


    println!("Soundbase listening on => {}", env_ip_str);
    warp::serve(api).run(sock_addr).await;
}

mod filters {
    use super::handlers;
    use warp::Filter;
    use std::sync::Arc;
    use crate::song_like::SourceMetadataDissect;

    type DB = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
    type Dissects = Arc<Vec<SourceMetadataDissect>>;

    pub fn endpoints(
        db: DB,
        dissects: Dissects,
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        heartbeat()
            .or(analytics_message(db.clone()))
            .or(song_fav(db.clone(), dissects))
            .or(fetch_tow(db.clone()))
            .or(fetch_aow(db))
    }

    pub fn heartbeat() -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("heartbeat")
            .and(warp::get())
            .and_then(handlers::heartbeat)
    }

    pub fn analytics_message(
        db: DB
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("analytics")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_db(db))
            .and_then(handlers::analytics_message)
    }

    pub fn song_fav(
        db: DB,
        dissects: Dissects,
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("song_fav")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_db(db))
            .and(with_dissects(dissects))
            .and_then(handlers::song_fav)
    }

    pub fn fetch_tow(
        db: DB
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("fetch" / "Top20OfWeek")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::fetch_top_20_of_week)
    }

    pub fn fetch_aow(
        db: DB
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("fetch" / "AlbumOfWeek")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::fetch_album_of_week)
    }

    fn with_db(db: DB) -> impl Filter<Extract=(DB, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn with_dissects(dissects: Dissects) -> impl Filter<Extract=(Dissects, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || dissects.clone())
    }
}

mod handlers {
    use std::sync::Arc;
    use http::StatusCode;
    use crate::{SongDB, top20_of_week, song_like_handler, analytics_handler};
    use crate::album_of_week;
    use crate::song_like::SourceMetadataDissect;

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
                let mut song_db = SongDB::new(&mut db_conn);
                let resp = song_like_handler::consume_like_message(&mut song_db, &dissects, body.to_vec());

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
                let mut song_db = SongDB::new(&mut db_conn);
                let response = top20_of_week::fetch_new_rockantenne_top20_of_week(&mut song_db);
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
                let mut song_db = SongDB::new(&mut db_conn);
                let response = album_of_week::fetch_new_rockantenne_album_of_week(&mut song_db);
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

    fn reply<T : warp::Reply>(r: T, status: StatusCode) -> impl warp::Reply {
        warp::reply::with_status(r, status)
    }
}