use r2d2_sqlite::SqliteConnectionManager;
use r2d2::{PooledConnection};
use rusqlite::{Rows, ToSql, Statement};
use crate::song_like::RawSong;
use crate::error::SoundbaseError;
use crate::song_like::{SongMetadata};
use rusqlite::types::ToSqlOutput;

type Result<R> = std::result::Result<R, SongDBError>;

pub struct SongDB<'a> {
    db: &'a mut PooledConnection<SqliteConnectionManager>,
}

#[derive(Default, Debug)]
pub struct Artist {
    id: u64,
    pub name: String,
    pub spot_id: String,
}

impl Artist {
    pub fn new(name: String, spot_id: String) -> Self {
        Artist {
            id: 0,
            name,
            spot_id,
        }
    }

    fn from_id_only(id: u64) -> Self {
        Artist {
            id,
            name: "".to_string(),
            spot_id: "".to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Album {
    id: u64,
    pub name: String,
    pub spot_id: String,
    artist_id: u64,
}

impl Album {
    pub fn new(name: String, spot_id: String, artist: Artist) -> Result<Self> {
        if artist.id == 0 {
            return Err(SongDBError::new("Provided Artist with id 0. This is not allowed! Store Artist first to obtain ID."))
        }
        Ok(Album {
            id: 0,
            name,
            spot_id,
            artist_id: artist.id,
        })
    }
}

#[derive(Default, Debug)]
pub struct Song {
    id: u64,
    pub name: String,
    pub is_faved: bool,
    pub spot_id: String,
    artist_id: u64,
    album_id: Option<u64>,
}

impl Song {
    pub fn new(name: String, spot_id: String, artist: Artist) -> Self {
        Song {
            id: 0,
            name,
            spot_id,
            artist_id: artist.id,
            album_id: None,
            is_faved: false,
        }
    }
}

#[derive(Debug)]
pub struct AlbumOfTheWeek {
    id: u64,
    album_id: u64,
    album_song_list_raw: String,
    source: String,
    source_comment: String,
    source_date: chrono::DateTime<chrono::FixedOffset>,
}

impl AlbumOfTheWeek {
    //TODO: return Result and check whether album id is valid
    pub fn new(source: String, source_comment: String, source_date: chrono::DateTime<chrono::FixedOffset>, album: Album, album_song_list_raw: String) -> Self {
        AlbumOfTheWeek {
            id: 0,
            source,
            source_comment,
            source_date,
            album_song_list_raw,
            album_id: album.id
        }
    }
}

impl PartialEq for AlbumOfTheWeek {
    fn eq(&self, other: &Self) -> bool {
        println!("Comparing two AofW...");
        println!("\tsource => {:?} <=> {:?} => {:?}", self.source, other.source, self.source == other.source);
        println!("\tdate => {:?} <=> {:?} => {:?}", self.source_date, other.source_date, self.source_date == other.source_date);
        println!("\talbum => {:?} <=> {:?} => {:?}", self.album_id, other.album_id, self.album_id == other.album_id);
        self.source == other.source && self.source_date == other.source_date && self.album_id == other.album_id
    }
}
impl Eq for AlbumOfTheWeek {}

#[derive(Debug)]
pub struct TopOfTheWeekEntry {
    id: u64,
    pub week: u8,
    pub year: u16,
    pub source: String,
    song_id: u64,
    pub chart_position: u8
}

impl TopOfTheWeekEntry {
    //TODO: return result and check whether song id is valid
    pub fn new(year: u16, week: u8, source: String, position: u8, song: Song) -> Self {
        TopOfTheWeekEntry {
            id: 0,
            year,
            week,
            source,
            chart_position: position,
            song_id: song.id
        }
    }
}

pub trait Load<R> {
    fn load(&mut self, id: u64) -> Result<R>;
}

pub trait FollowForeignReference<O, D> {
    fn follow_reference(&mut self, to_follow: &O) -> Result<D>;
}

pub trait FindUnique<R, Q> {
    fn find_unique(&mut self, query: &Q) -> Result<Option<R>>;
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

pub trait SongHistDB {
    fn store_song_hist_entry(&mut self, song: &Song, meta: &SongMetadata, new_state: bool) -> Result<()>;
}

impl<'a, 'f> SongDB<'a> {
    pub fn new(db: &'a mut PooledConnection<SqliteConnectionManager>) -> Self {
        SongDB { db }
    }

    pub fn create_song_from_raw(&mut self, raw: &'f RawSong) -> Result<Song> {
        let mut song = Song::default();

        //1. check for existing artist
        let find = FindArtist(raw.artist.as_str());
        let artist_option = self.find_unique(&find)?;
        song.artist_id = match artist_option {
            Some(a) => a.id,
            None => {
                let mut artist = Artist::new(raw.artist.clone(), "".to_string());
                self.save(&mut artist)?;
                artist.id
            }
        };

        //2. is an album supplied?
        if !raw.album.is_empty() {
            let artist = Artist::from_id_only(song.artist_id);
            let find = FindAlbum(raw.album.as_str(), &artist);
            let album_option = self.find_unique(&find)?;
            song.album_id = match album_option {
                Some(a) => Some(a.id),
                None => {
                    let mut album = Album::new(raw.album.clone(), "".to_string(), Artist::from_id_only(song.artist_id))?;
                    self.save(&mut album)?;
                    Some(album.id)
                }
            };
        }

        //3. finally create the song
        song.name = raw.title.clone();
        song.spot_id = "".to_string();

        self.save(&mut song)?;

        Ok(song)
    }
}

impl<'a> SongHistDB for SongDB<'a> {
    fn store_song_hist_entry(&mut self, song: &Song, meta: &SongMetadata, new_state: bool) -> Result<()> {
        println!("\tStored in Hist DB!");

        let mut stmt = self.db.prepare("INSERT INTO song_history (song_id,change_origin,change_source,change_name,prev_state,new_state) VALUES(?,?,?,?,?,?)")?;
        stmt.execute(rusqlite::params![
            song.id,
            &meta.origin,
            &meta.source.source_kind,
            &meta.source.source_name,
            !new_state,
            new_state
        ])?;

        Ok(())
    }
}

impl<'a> Load<Artist> for SongDB<'a> {
    fn load(&mut self, id: u64) -> Result<Artist> {
        if id == 0 {
            Err(SongDBError::new("Invalid ID given!"))
        }else {
            let mut prep_stmt = self.db.prepare("SELECT artist_name,artist_spot_id FROM artists WHERE artist_id = ? LIMIT 1")?;
            let mut rows = prep_stmt.query([
                id
            ])?;

            match rows.next()? {
                Some(row) => {
                    Ok(Artist {
                        id,
                        name: row.get(0)?,
                        spot_id: row.get(1)?,
                    })
                }

                None => {
                    Err(SongDBError::new("Didn't find the artist for the given artist_id!"))
                }
            }
        }
    }
}

impl<'a> Load<Album> for SongDB<'a> {
    fn load(&mut self, id: u64) -> Result<Album> {
        if id == 0 {
            Err(SongDBError::new("Invalid ID given!"))
        }else {
            let mut prep_stmt = self.db.prepare("SELECT album_name, album_spot_id, artist_id FROM albums WHERE album_id = ? LIMIT 1")?;
            let mut rows = prep_stmt.query([
                id
            ])?;
            match rows.next()? {
                Some(row) => {
                    Ok(Album {
                        id,
                        name: row.get(0)?,
                        spot_id: row.get(1)?,
                        artist_id: row.get(2)?,
                    })
                }
                None => Err(SongDBError::new("Didn't find the album for the given album_id!"))
            }
        }
    }
}

impl<'a> Load<Song> for SongDB<'a> {
    fn load(&mut self, id: u64) -> Result<Song> {
        if id == 0 {
            Err(SongDBError::new("Invalid ID given!"))
        }else {
            let mut prep_stmt = self.db.prepare("SELECT song_title,song_spot_id,is_faved,artist_id,album_id FROM songs WHERE song_id = ? LIMIT 1")?;
            let mut rows = prep_stmt.query([
                id
            ])?;
            match rows.next()? {
                Some(row) => {
                    Ok(Song {
                        id,
                        name: row.get(0)?,
                        spot_id: row.get(1)?,
                        is_faved: row.get(2)?,
                        artist_id: row.get(3)?,
                        album_id: row.get(4)?,
                    })
                }
                None => Err(SongDBError::new("Didn't find the song for the given song_id!"))
            }
        }
    }
}

impl<'a> Load<AlbumOfTheWeek> for SongDB<'a> {
    fn load(&mut self, id: u64) -> Result<AlbumOfTheWeek> {
        if id == 0 {
            Err(SongDBError::new("Invaild ID given!"))
        }else {
            let mut stmt = self.db.prepare("SELECT album_id,album_song_list_raw,source_name,source_comment,source_date FROM albums_of_week WHERE week_id = ? LIMIT 1")?;
            let mut rows = stmt.query(rusqlite::params![id])?;

            match rows.next()? {
                Some(row) => {
                    let tmstp : String = row.get(4)?;
                    let d = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&tmstp)?;

                    Ok(AlbumOfTheWeek{
                        id,
                        album_id: row.get(0)?,
                        album_song_list_raw: row.get(1)?,
                        source: row.get(2)?,
                        source_comment: row.get(3)?,
                        source_date: d
                    })
                },
                None => Err(SongDBError::new("Didn't find the album of the week for the given week id!"))
            }
        }
    }
}

impl<'a> FollowForeignReference<Album, Artist> for SongDB<'a> {
    fn follow_reference(&mut self, to_follow: &Album) -> Result<Artist> {
        self.load(to_follow.artist_id)
    }
}

impl<'a> FollowForeignReference<Song, Album> for SongDB<'a> {
    fn follow_reference(&mut self, to_follow: &Song) -> Result<Album> {
        if to_follow.album_id.is_some() {
            self.load(to_follow.album_id.unwrap())
        } else {
            Err(SongDBError::new("Song isn't assigned to an album!"))
        }
    }
}

impl<'a> FollowForeignReference<Song, Artist> for SongDB<'a> {
    fn follow_reference(&mut self, to_follow: &Song) -> Result<Artist> {
        self.load(to_follow.artist_id)
    }
}

impl<'a> FollowForeignReference<AlbumOfTheWeek, Album> for SongDB<'a> {
    fn follow_reference(&mut self, to_follow: &AlbumOfTheWeek) -> Result<Album> {
        self.load(to_follow.album_id)
    }
}

pub struct FindArtist<'a>(&'a str);
impl<'a> FindArtist<'a> {
    pub fn new(name: &'a str) -> Self {
        FindArtist(name)
    }
}
impl<'a, 'f> FindUnique<Artist, FindArtist<'f>> for SongDB<'a> {
    fn find_unique(&mut self, query: &FindArtist<'f>) -> Result<Option<Artist>> {
        let mut stmt = self.db.prepare("SELECT * FROM artists WHERE artist_name = ? LIMIT 1")?;
        let mut rows = stmt.query(rusqlite::params![query.0])?;

        match rows.next()? {
            Some(row) => Ok(Some(Artist { id: row.get(0)?, name: row.get(1)?, spot_id: row.get(2)? })),
            None => Ok(None)
        }
    }
}

pub struct FindAlbum<'a>(&'a str, &'a Artist);
impl<'a> FindAlbum<'a> {
    pub fn new(name: &'a str, artist: &'a Artist) -> Self {
        FindAlbum(name, artist)
    }
}

impl<'a, 'f> FindUnique<Album, FindAlbum<'f>> for SongDB<'a> {
    fn find_unique(&mut self, query: &FindAlbum<'f>) -> Result<Option<Album>> {
        assert_ne!(query.1.id, 0);
        let mut stmt = self.db.prepare("SELECT * FROM albums WHERE album_name = ? AND artist_id = ? LIMIT 1")?;
        let mut rows = stmt.query(rusqlite::params![query.0, query.1.id])?;

        match rows.next()? {
            Some(row) => Ok(Some(Album {
                id: row.get(0)?,
                artist_id: row.get(1)?,
                name: row.get(2)?,
                spot_id: row.get(3)?,
            })),
            None => Ok(None)
        }
    }
}

pub struct FindSong<'a>(&'a str, &'a Artist, Option<&'a Album>);
impl<'a> FindSong<'a> {
    pub fn new(name: &'a str, artist: &'a Artist, album: Option<&'a Album>) -> Self {
        FindSong(name, artist, album)
    }
}
impl<'a, 'f> FindUnique<Song, FindSong<'f>> for SongDB<'a> {
    fn find_unique(&mut self, query: &FindSong<'f>) -> Result<Option<Song>> {
        assert_ne!(query.1.id, 0);
        assert!(query.2.is_some() && query.2.unwrap().id != 0 || query.2.is_none());

        let mut q = "SELECT * FROM songs WHERE song_title = ? AND artist_id = ? AND album_id ".to_string();
        q += if query.2.is_some() { "= ?" } else { "IS ?"};
        q += " LIMIT 1";

        let mut stmt = self.db.prepare(&q)?;
        let mut rows = stmt.query(rusqlite::params![query.0, query.1.id, query.2])?;

        match rows.next()? {
            Some(row) => Ok(Some(Song {
                id: row.get(0)?,
                name: row.get(1)?,
                spot_id: row.get(2)?,
                artist_id: row.get(3)?,
                album_id: row.get(4)?,
                is_faved: row.get(5)?,
            })),
            None => Ok(None)
        }
    }
}

impl<'a> FindUnique<Song, RawSong> for SongDB<'a> {
    fn find_unique(&mut self, query: &RawSong) -> Result<Option<Song>> {
        let song_id: u64 = {
            let mut stmt: Statement;
            let mut result: Rows;

            if !query.album.is_empty() {
                stmt = self.db.prepare("SELECT song_id FROM songs,artists,albums WHERE songs.artist_id = artists.artist_id AND songs.album_id = albums.album_id \
                AND songs.song_title = ? AND artists.artist_name = ? AND albums.album_name = ? LIMIT 1")?;
                result = stmt.query(rusqlite::params![query.title, query.artist, query.album])?;
            } else {
                stmt = self.db.prepare("SELECT song_id FROM songs,artists WHERE songs.artist_id = artists.artist_id \
                AND songs.song_title = ? AND artists.artist_name = ? LIMIT 1")?;
                result = stmt.query(rusqlite::params![query.title, query.artist])?;
            }

            match result.next()? {
                Some(row) => {
                    let id: u64 = row.get(0)?;
                    id
                }
                None => return Ok(None)
            }
        };

        Ok(Some(self.load(song_id)?))
    }
}

pub struct AlbumOfTheWeekQuery();
impl<'a> Query<AlbumOfTheWeek, AlbumOfTheWeekQuery> for SongDB<'a> {
    fn query(&mut self, bounds: QueryBounds, _filter: Option<AlbumOfTheWeekQuery>, ordering: Option<QueryOrdering>) -> Result<Vec<AlbumOfTheWeek>> {
        let query : String = {
            let mut base = "SELECT * FROM albums_of_week".to_string();
            match ordering {
                Some(order) => {
                    base += " ORDER BY ";
                    base += order.on_field.as_str();
                    base += " ";
                    base += match order.direction { OrderDirection::Asc => "ASC", OrderDirection::Desc => "DESC" };
                },
                None => {}
            };

            base += " LIMIT ? OFFSET ?";
            base
        };
        let mut stmt = self.db.prepare(query.as_str())?;
        let mut rows = stmt.query(rusqlite::params![bounds.page_size, bounds.offset])?;
        let mut result = Vec::<AlbumOfTheWeek>::new();
        while let Some(row) = rows.next()? {
            let dt_str :String = row.get(5)?;
            let tmstp = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&dt_str)?;
            result.push(AlbumOfTheWeek{
                id: row.get(0)?,
                album_id: row.get(1)?,
                album_song_list_raw: row.get(2)?,
                source: row.get(3)?,
                source_comment: row.get(4)?,
                source_date: tmstp
            });
        }
        Ok(result)
    }
}


impl<'a> Save<Artist> for SongDB<'a> {
    fn save(&mut self, to_save: &mut Artist) -> Result<()> {
        if to_save.id == 0 {
            //do insert
            let result : usize = {
                let mut stmt = self.db.prepare("INSERT INTO artists (artist_name,artist_spot_id) VALUES(?,?)")?;
                stmt.execute(rusqlite::params![to_save.name, to_save.spot_id])?
            };
            if result == 1 {
                //fetch the new artist ID
                to_save.id = last_row_id(&mut self.db)?;
                Ok(())
            } else {
                Err(SongDBError::new("Failed to create new artist entry!"))
            }
        } else {
            //do update
            let mut stmt = self.db.prepare("UPDATE artists SET artist_name = ?, artist_spot_id = ? WHERE artist_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.id])?;

            if result != 1 {
                Err(SongDBError::new("Failed to update the given artist!"))
            }else{
                Ok(())
            }
        }
    }
}

impl<'a> Save<Album> for SongDB<'a> {
    fn save(&mut self, to_save: &mut Album) -> Result<()> {
        debug_assert!(to_save.artist_id != 0);
        if to_save.id == 0 {
            //Do insert
            let result = {
                let mut stmt = self.db.prepare("INSERT INTO albums (album_name,album_spot_id,artist_id) VALUES(?,?,?)")?;
                stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id])?
            };
            if result == 1 {
                to_save.id = last_row_id(&mut self.db)?;
                Ok(())
            } else {
                Err(SongDBError::new("Failed to create new album with the given data!"))
            }
        } else {
            //Do a update
            let mut stmt = self.db.prepare("UPDATE albums SET album_name = ?, album_spot_id = ?, artist_id = ? WHERE album_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id, to_save.id])?;

            if result != 1 {
                Err(SongDBError::new("Failed to update the given album!"))
            }else{
                Ok(())
            }
        }
    }
}

impl<'a> Save<Song> for SongDB<'a> {
    fn save(&mut self, to_save: &mut Song) -> Result<()> {
        debug_assert!(to_save.artist_id != 0);
        if to_save.id == 0 {
            //Do isert
            let result : usize = {
                let mut stmt = self.db.prepare("INSERT INTO songs (song_title, song_spot_id, artist_id, album_id, is_faved) VALUES(?,?,?,?,?)")?;
                stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id, to_save.album_id, to_save.is_faved])?
            };
            if result == 1 {
                to_save.id = last_row_id(&mut self.db)?;
                Ok(())
            } else {
                Err(SongDBError::new("Failed to create new song with the given data!"))
            }
        } else {
            //Do update
            let mut stmt = self.db.prepare("UPDATE songs SET song_title = ?, song_spot_id = ?, artist_id = ?, album_id = ?, is_faved = ? WHERE song_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id, to_save.album_id, to_save.is_faved, to_save.id])?;

            if result != 1 {
                Err(SongDBError::new("Failed to update the given song!"))
            }else{
                Ok(())
            }
        }
    }
}

impl<'a> Save<AlbumOfTheWeek> for SongDB<'a> {
    fn save(&mut self, to_save: &mut AlbumOfTheWeek) -> Result<()> {
        debug_assert!(to_save.album_id != 0);
        let date_time = to_save.source_date.to_rfc3339();

        if to_save.id == 0 {
            //do insert
            let result: usize = {
                let mut stmt = self.db.prepare("INSERT INTO albums_of_week (album_id,album_song_list_raw,source_name,source_comment,source_date) VALUES(?,?,?,?,?)")?;
                stmt.execute(rusqlite::params![to_save.album_id, to_save.album_song_list_raw, to_save.source, to_save.source_comment, date_time])?
            };
            if result == 1 {
                to_save.id = last_row_id(&mut self.db)?;
                Ok(())
            }else {
                Err(SongDBError::new("Failed to create new album of week entry with given data!"))
            }
        }else{
            //Do update
            let mut stmt = self.db.prepare("UPDATE albums_of_week SET album_id = ?, album_song_list_raw = ?, source_name = ?, source_comment = ?, source_date = ? WHERE week_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.album_id, to_save.album_song_list_raw, to_save.source, to_save.source_comment, date_time, to_save.id])?;

            if result != 1 {
                Err(SongDBError::new("Failed to update the given album of week!"))
            }else {
                Ok(())
            }
        }
    }
}

impl<'a> Save<TopOfTheWeekEntry> for SongDB<'a> {
    fn save(&mut self, to_save: &mut TopOfTheWeekEntry) -> Result<()> {
        debug_assert!(to_save.song_id != 0);

        if to_save.id == 0 {
            //Do insert
            let result: usize = {
                let mut stmt = self.db.prepare("INSERT INTO top_charts_of_week (calendar_week,year,source_name,song_id,song_position) VALUES(?,?,?,?,?)")?;
                stmt.execute(rusqlite::params![to_save.week, to_save.year, to_save.source, to_save.song_id, to_save.chart_position])?
            };
            if result == 1 {
                to_save.id = last_row_id(&mut self.db)?;
                Ok(())
            }else{
                Err(SongDBError::new("Failed to ceate new top chart of the week entry with given data!"))
            }
        }else {
            //Do Update
            let mut stmt = self.db.prepare("UPDATE top_charts_of_week SET calendar_week = ?, year = ?, source_name = ?, song_id = ?, song_position = ? WHERE week_song_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.week, to_save.year, to_save.source, to_save.song_id, to_save.chart_position, to_save.id])?;

            if result != 1 {
                Err(SongDBError::new("Failed to update top chart of week entry with given data!"))
            }else{
                Ok(())
            }
        }
    }
}

fn last_row_id(db: &mut PooledConnection<SqliteConnectionManager>) -> Result<u64> {
    let mut prep_stmt = db.prepare("SELECT last_insert_rowid()")?;
    let mut rows = prep_stmt.query(rusqlite::params![])?;
    match rows.next()? {
        Some(row) => {
            let id : u64 = row.get(0)?;
            Ok(id)
        },
        None => Err(SongDBError::new("Failed to receive new last insert rowid!"))
    }
}

impl<'a> Delete<Artist> for SongDB<'a> {
    fn delete(&mut self, to_delete: &Artist) -> Result<()> {
        delete(&mut self.db, "artists", "artist_id", to_delete.id)
    }
}

impl<'a> Delete<Album> for SongDB<'a> {
    fn delete(&mut self, to_delete: &Album) -> Result<()> {
        delete(&mut self.db, "albums", "album_id", to_delete.id)
    }
}

impl<'a> Delete<Song> for SongDB<'a> {
    fn delete(&mut self, to_delete: &Song) -> Result<()> {
        delete(&mut self.db, "songs", "song_id", to_delete.id)
    }
}

impl<'a> Delete<AlbumOfTheWeek> for SongDB<'a> {
    fn delete(&mut self, to_delete: &AlbumOfTheWeek) -> Result<()> {
        delete(&mut self.db, "albums_of_week", "week_id", to_delete.id)
    }
}

fn delete(db: &mut PooledConnection<SqliteConnectionManager>, table: &'static str, id_field: &'static str, id: u64) -> Result<()> {
    if id == 0 {
        Err(SongDBError::new("Can't delete row with ID 0!"))
    }else {
        let mut prep_stmt = db.prepare(("DELETE FROM ".to_owned() + table + " WHERE " + id_field + " = ?").as_str())?;
        let result = prep_stmt.execute([id])?;

        if result != 1 {
            Err(SongDBError::new("Failed to delete row from table"))
        }else{
            Ok(())
        }
    }
}

pub struct SongDBError(String);
impl SongDBError {
    fn new(msg: &str) -> Self {
        SongDBError(msg.to_string())
    }
}

impl From<SongDBError> for SoundbaseError {
    fn from(e: SongDBError) -> Self {
        SoundbaseError {
            msg: e.0.to_string(),
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<rusqlite::Error> for SongDBError {
    fn from(e: rusqlite::Error) -> Self {
        let msg = format!("{:?}", e);
        SongDBError(msg)
    }
}

impl From<chrono::ParseError> for SongDBError {
    fn from(e: chrono::ParseError) -> Self {
        SongDBError(e.to_string())
    }
}

impl ToSql for Album {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.id as i64))
    }
}