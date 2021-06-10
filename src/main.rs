use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

mod error;
pub mod model;
pub mod db;
pub mod handlers;
pub mod generated;

#[derive(Clone)]
struct RequestPayload {
    db_pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    dissects: std::sync::Arc<model::song_like::SourceMetadataDissectConfig>,
}

#[tokio::main]
async fn main() {
    let db = db::initialize_db().expect("Failed to create DB!");

    let metadata_dissect = model::song_like::SourceMetadataDissectConfig::load_from_file("./config.json");
    println!("Read the following Metadata dissects:");
    println!("\t{:?}", metadata_dissect);
    println!();

    let mut spotify = crate::model::spotify::Spotify::new();
    match spotify.finish_initialization_from_cache().await {
        Ok(_) => println!("Spotify access enabled."),
        Err(e) =>
            println!("Couldn't load spotif access token from cache (Error: {:?}). Consider authenticating by calling /spotify/start_auth.", e)
    }

    let api = filters::endpoints(db, Arc::new(metadata_dissect.sources), Arc::new(RwLock::new(spotify)));


    let env_ip_str = match std::env::var("SERVER_IP") {
        Ok(given_ip) => given_ip,
        Err(_) => "192.168.2.111:3333".to_string()
    };
    let sock_addr: SocketAddr = env_ip_str.parse().unwrap();


    println!("Soundbase listening on => {}", env_ip_str);
    warp::serve(api).run(sock_addr).await;
}

mod filters {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;
    use warp::Filter;

    use crate::model::song_like::SourceMetadataDissect;
    use super::handlers;
    use crate::model::spotify::Spotify;

    type DB = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
    type Dissects = Arc<Vec<super::model::song_like::SourceMetadataDissect>>;

    pub fn endpoints(
        db: DB,
        dissects: Dissects,
        spotify: Arc<RwLock<Spotify>>,
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        heartbeat()
            .or(analytics_message(db.clone()))
            .or(song_fav(db.clone(), dissects))
            .or(fetch_tow(db.clone()))
            .or(fetch_aow(db))
            .or(spotify_start_authorization(spotify.clone()))
            .or(spotify_auth_callback(spotify.clone()))
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

    pub fn spotify_start_authorization(
        spotify: Arc<RwLock<Spotify>>
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("spotify" / "start_auth")
            .and(warp::get())
            .and(with_spotify(spotify))
            .and_then(handlers::spotify_start_auth)
    }

    pub fn spotify_auth_callback(
        spotify: Arc<RwLock<Spotify>>
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("spotify" / "auth_callback")
            .and(warp::get())
            .and(with_spotify(spotify))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(handlers::spotify_auth_callback)
    }

    fn with_db(db: DB) -> impl Filter<Extract=(DB, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn with_dissects(dissects: Dissects) -> impl Filter<Extract=(Dissects, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || dissects.clone())
    }

    fn with_spotify(spot: Arc<RwLock<Spotify>>) -> impl Filter<Extract=(Arc<RwLock<Spotify>>, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || spot.clone())
    }
}