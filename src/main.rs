mod error;
mod db;
mod analytics;
mod analytics_handler;
mod analytics_protocol_generated;
mod song_like;
mod song_like_handler;
mod song_like_protocol_generated;

#[async_std::main]
async fn main() -> tide::Result<()> {

    let db = db::setup_db().expect("Failed to create DB!");
    let mut app = tide::with_state(db);


    app.at("/analytics").post(|mut req: tide::Request<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>| async move {
        let body = req.body_bytes().await?;
        let mut db = req.state().get()?;
        let _ = analytics_handler::consume_analytics_message(&mut db, body);
        Ok(tide::Response::new(tide::StatusCode::Accepted))
    });

    app.at("/song_fav").post(|mut req: tide::Request<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>| async move {
        let body = req.body_bytes().await?;
        let mut db = req.state().get()?;
        let response = song_like_handler::consume_like_message(&mut db, body);

        match response {
            Ok(r) => Ok(tide::Response::builder(tide::StatusCode::Ok).body(r).build()),
            Err(e) => Ok(tide::Response::builder(e.http_code).body(e.msg).build())
        }
    });

    app.listen("192.168.2.101:3333").await?;

    Ok(())
}