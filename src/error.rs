use std::string::FromUtf8Error;
use std::num::ParseIntError;

pub type Result<T, E = SoundbaseError> = core::result::Result<T, E>;

#[derive(Debug)]
pub struct SoundbaseError {
    pub http_code: http::StatusCode,
    pub msg: String,
}

impl SoundbaseError {
    pub fn new(msg: &'static str) -> Self {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: msg.to_string()
        }
    }
}

impl warp::Reply for SoundbaseError {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(self.msg, self.http_code).into_response()
    }
}

impl From<r2d2::Error> for SoundbaseError {
    fn from(e: r2d2::Error) -> SoundbaseError {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string(),
        }
    }
}

impl From<rusqlite::Error> for SoundbaseError {
    fn from(e: rusqlite::Error) -> SoundbaseError {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<regex::Error> for SoundbaseError {
    fn from(e: regex::Error) -> Self {
        SoundbaseError{
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<std::io::Error> for SoundbaseError {
    fn from(e: std::io::Error) -> Self {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<serde_json::Error> for SoundbaseError {
    fn from(e: serde_json::Error) -> Self {
        SoundbaseError{
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<reqwest::Error> for SoundbaseError {
    fn from(e: reqwest::Error) -> Self {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<chrono::ParseError> for SoundbaseError {
    fn from(e: chrono::ParseError) -> Self {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<FromUtf8Error> for SoundbaseError {
    fn from(e: FromUtf8Error) -> Self {
        SoundbaseError {
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}

impl From<ParseIntError> for SoundbaseError {
    fn from(e: ParseIntError) -> Self {
        SoundbaseError{
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            msg: e.to_string()
        }
    }
}