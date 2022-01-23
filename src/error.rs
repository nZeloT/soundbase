/*
 * Copyright 2021 nzelot<leontsteiner@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::convert::Infallible;
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("spotify client error: {0}")]
    SpotifyError(#[from] rspotify::ClientError),

    #[error("spotify id error: {0}")]
    SpotifyIdError(#[from] rspotify::model::IdError),

    #[error("spotify model error: {0}")]
    SpotifyModelError(#[from] rspotify::model::ModelError),

    #[error("database error: {0}")]
    DatabaseError(#[from] super::db_new::DbError),

    #[error("api error: {0}")]
    ApiError(String),

    #[error("client request error: {0}")]
    RequestError(String),

    #[error("internal error: {0}")]
    InternalError(String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Int parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("Chrono parse error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),

    #[error("input/output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Utf8 parse error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Reqwest error: {0}")]
    RequwestError(#[from] reqwest::Error)
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub status: String,
    pub kind : String,
    pub message: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_rejection(err : warp::Rejection) -> std::result::Result<impl warp::Reply, Infallible> {
    let (code, kind, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "404".to_string(), "not found".to_string())
    }else if let Some(e) = err.find::<Error>() {
        match e {
            Error::SpotifyError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Spotify".to_string(), n.to_string()),
            Error::SpotifyIdError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Spotify".to_string(), n.to_string()),
            Error::SpotifyModelError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Spotify".to_string(), n.to_string()),
            Error::ApiError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "API".to_string(), n.clone()),
            Error::RequestError(n) => (StatusCode::BAD_REQUEST, "Client".to_string(), n.clone()),
            Error::InternalError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal".to_string(), n.clone()),
            Error::DatabaseError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Database".to_string(), n.to_string()),
            Error::RegexError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Regex".to_string(), n.to_string()),
            Error::SerdeError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Serde".to_string(), n.to_string()),
            Error::ParseError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Parse".to_string(), n.to_string()),
            Error::ChronoParseError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Chrono Parse".to_string(), n.to_string()),
            Error::IoError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "IO".to_string(), n.to_string()),
            Error::Utf8Error(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Utf8 Parse".to_string(), n.to_string()),
            Error::RequwestError(n) => (StatusCode::INTERNAL_SERVER_ERROR, "Requwest".to_string(), n.to_string())
        }
    }else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (StatusCode::METHOD_NOT_ALLOWED, "HTTP".to_string(), "Method not allowed!".to_string())
    }else{
        eprintln!("unhandled error {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Unknown".to_string(), "Internal Server Error".to_string())
    };

    let json = warp::reply::json(&ErrorResponse{
        status: code.to_string(),
        kind,
        message
    });

    Ok(warp::reply::with_status(json, code))
}