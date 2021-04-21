pub type Result<T, E = SoundbaseError> = core::result::Result<T, E>;

#[derive(Debug)]
pub struct SoundbaseError {
    pub http_code: tide::StatusCode,
    pub msg: String,
}

impl From<r2d2::Error> for SoundbaseError {
    fn from(e: r2d2::Error) -> SoundbaseError {
        SoundbaseError {
            http_code: tide::StatusCode::InternalServerError,
            msg: e.to_string(),
        }
    }
}

impl From<rusqlite::Error> for SoundbaseError {
    fn from(e: rusqlite::Error) -> SoundbaseError {
        SoundbaseError {
            http_code: tide::StatusCode::InternalServerError,
            msg: e.to_string()
        }
    }
}