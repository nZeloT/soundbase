use warp::{Filter, http::StatusCode};
use bytes::{Buf};

mod analytics;

fn map_analytics_body(body: bytes::Bytes) -> StatusCode {
    analytics::consume_analytics_message(body.chunk());
    warp::http::StatusCode::ACCEPTED
}


#[tokio::main]
async fn main() {
    let hello = warp::get()
        .and(warp::path("hello"))
        .map(|| "Hello World from the Rust Soundbase");

    let analytics = warp::post()
        .and(warp::path("analytics"))
        .and(warp::body::content_length_limit(8096))
        .and(warp::body::bytes())
        .map(map_analytics_body);

    warp::serve(hello.or(analytics)).run(([192, 168, 2, 101], 2222)).await;
}