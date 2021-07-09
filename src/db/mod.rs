pub mod db_error;
pub mod artist;
pub mod album;
pub mod album_of_week;
pub mod song;
pub mod top_of_the_week;
mod song_history;
mod util;

type Result<R> = std::result::Result<R, db_error::DbError>;
type DB    = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub trait Load<R> {
    fn load(&mut self, id: u64) -> Result<R>;
}

pub trait FollowForeignReference<O, D> {
    fn follow_reference(&mut self, to_follow: &O) -> Result<D>;
}

pub trait FindUnique<R, Q> {
    fn find_unique(&mut self, query: Q) -> Result<Option<R>>;
}

pub enum OrderDirection {
    Asc,
    Desc,
}

pub struct QueryOrdering {
    pub direction: OrderDirection,
    pub on_field: String
}

pub struct QueryBounds {
    pub offset: u64,
    pub page_size: u16,
}

pub trait Query<R, P> {
    fn query(&mut self, bounds: QueryBounds, filter: Option<P>, ordering: Option<QueryOrdering>) -> Result<Vec<R>>;
}

pub trait Save<R> {
    fn save(&mut self, to_save: &mut R) -> Result<()>;
}

pub trait Delete<R> {
    fn delete(&mut self, to_delete: &R) -> Result<()>;
}

pub fn initialize_db() -> Result<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file("./soundbase.db");
    let pool = r2d2::Pool::new(manager)?;
    let conn = pool.get()?;
    conn.execute_batch("\
        CREATE TABLE IF NOT EXISTS artists (
            artist_id       INTEGER PRIMARY KEY AUTOINCREMENT,
            artist_name     VARCHAR(30) NOT NULL,
            artist_spot_id  VARCHAR(22),

            CONSTRAINT artist_unique UNIQUE(artist_name)
        );
        CREATE TABLE IF NOT EXISTS albums (
            album_id        INTEGER PRIMARY KEY AUTOINCREMENT,
            artist_id       INTEGER NOT NULL,
            album_name      VARCHAR(30) NOT NULL,
            album_spot_id   VARCHAR(22),

            FOREIGN KEY(artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS songs (
            song_id         INTEGER PRIMARY KEY AUTOINCREMENT,
            song_title      VARCHAR(20) NOT NULL,
            song_spot_id    VARCHAR(22),
            artist_id       INTEGER NOT NULL,
            album_id        INTEGER REFERENCES albums(album_id) ON DELETE CASCADE,
            is_faved        BOOLEAN,

            FOREIGN KEY(artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE,
            CONSTRAINT song_unique UNIQUE(song_title,artist_id,album_id)
        );
        CREATE TABLE IF NOT EXISTS song_history (
            hist_id        INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id        INTEGER NOT NULL,
            change_origin  VARCHAR(20) NOT NULL,
            change_source  TINYINT NOT NULL,
            change_name    VARCHAR(20) NOT NULL,
            prev_state     TINYINT,
            new_state      TINYINT,

            FOREIGN KEY(song_id) REFERENCES songs(song_id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS albums_of_week (
            week_id         INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id        INTEGER NOT NULL,
            album_song_list_raw TEXT,
            source_name     VARCHAR(20) NOT NULL,
            source_comment  TEXT,
            source_date     VARCHAR(40),
            FOREIGN KEY(album_id) REFERENCES albums(album_id)
        );
        CREATE TABLE IF NOT EXISTS top_charts_of_week (
            week_song_id    INTEGER PRIMARY KEY AUTOINCREMENT,
            calendar_week   TINYINT,
            year            TINYINT,
            source_name     VARCHAR(20) NOT NULL,
            song_id         INTEGER NOT NULL,
            song_position   TINYINT,

            FOREIGN KEY(song_id) REFERENCES songs(song_id)
        );
    ")?;
    Ok(pool)
}