#[derive(Debug)]
pub struct DbError(String);
impl DbError {
    pub fn new(msg: &str) -> Self {
        DbError(msg.to_string())
    }
}

impl From<DbError> for crate::error::SoundbaseError {
    fn from(e: DbError) -> Self {
        crate::error::SoundbaseError {
            msg: e.0.to_string(),
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        let msg = format!("{:?}", e);
        DbError(msg)
    }
}

impl From<chrono::ParseError> for DbError {
    fn from(e: chrono::ParseError) -> Self {
        DbError(e.to_string())
    }
}

impl From<r2d2::Error> for DbError {
    fn from(e: r2d2::Error) -> Self {
        DbError(e.to_string())
    }
}