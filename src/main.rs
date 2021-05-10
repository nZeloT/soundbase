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

#[async_std::main]
async fn main() -> tide::Result<()> {
    let db = db::setup_db().expect("Failed to create DB!");

    let metadata_dissect = song_like::SourceMetadataDissectConfig::load_from_file("./config.json");
    println!("Read the following Metadata dissects:");
    println!("\t{:?}", metadata_dissect);
    println!();

    let payload = RequestPayload {
        db_pool: db,
        dissects: std::sync::Arc::new(metadata_dissect),
    };


    let mut app = tide::with_state(payload);

    app.at("/analytics").post(|mut req: tide::Request<RequestPayload>| async move {
        let body = req.body_bytes().await?;
        let payload = req.state();
        let mut db = payload.db_pool.get()?;
        let _ = analytics_handler::consume_analytics_message(&mut db, body);
        Ok(tide::Response::new(tide::StatusCode::Accepted))
    });

    app.at("/analytics/heartbeat").get(|_| async move {
       //it's a heartbeat so just respond with a static Ok
        println!("Received a Heartbeat request for analytics.");
        println!();
        Ok(tide::Response::new(tide::StatusCode::Ok))
    });

    app.at("/song_fav").post(|mut req: tide::Request<RequestPayload>| async move {
        let body = req.body_bytes().await?;

        let payload = req.state();
        let mut db = payload.db_pool.get()?;
        let mut song_db = song_db::SongDB::new(&mut db);
        let response = song_like_handler::consume_like_message(&mut song_db, &payload.dissects.sources, body);

        match response {
            Ok(r) => Ok(tide::Response::builder(tide::StatusCode::Ok).body(r).build()),
            Err(e) => {
                println!("\tRssponding with Error => {:?}", e.msg);
                Ok(tide::Response::builder(e.http_code).body(e.msg).build())
            }
        }
    });

    app.at("/song_fav/heartbeat").get(|_| async move {
        //it's a heartbeat so just respond with a static Ok
        println!("Received a Heartbeat request for song_fav.");
        println!();
        Ok(tide::Response::new(tide::StatusCode::Ok))
    });

    app.at("/fetch").get(|mut req: tide::Request<RequestPayload>| async move {
        let payload = req.state();
        let mut db = payload.db_pool.get()?;
        let mut song_db = song_db::SongDB::new(&mut db);
        let response = album_of_week::fetch_new_rockantenne_album_of_week(&mut song_db);
        match response {
            Ok(..) => Ok(tide::Response::builder(tide::StatusCode::Ok).build()),
            Err(e) => {
                println!("\tRssponding with Error => {:?}", e.msg);
                Ok(tide::Response::builder(e.http_code).body(e.msg).build())
            }
        }
    });

    app.listen("192.168.2.101:3333").await?;

    Ok(())
}