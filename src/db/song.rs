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

use super::{Result, DbPool, Load, FindUnique, Save, Delete, FollowForeignReference, db_error::DbError};
use super::util::{last_row_id, delete};
use super::artist::{Artist, FindArtist};
use super::album::{Album, FindAlbum};

use crate::model::song_like::{RawSong, SongMetadata, SongState};
#[derive(Default, Debug)]
pub struct Song {
    pub(super) id: u64,
    pub name: String,
    pub is_faved: bool,
    pub spot_id: String,
    pub(super) artist_id: u64,
    pub(super) album_id: Option<u64>,
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

    pub fn has_album_info(&self) -> bool {
        self.album_id != None
    }
}

pub fn create_song_from_raw<DB>(db: &DB, raw: &RawSong) -> Result<Song>
    where DB: FindUnique<Artist, FindArtist> + FindUnique<Album, FindAlbum> + Save<Artist> + Save<Album> + Save<Song>
{
    let mut song = Song::default();

    //1. check for existing artist

    let artist = {
        let artist_option = db.find_unique(FindArtist::new(raw.artist.clone()))?;
        match artist_option {
            Some(a) => a,
            None => {
                let mut artist = Artist::new(raw.artist.clone(), "".to_string());
                db.save(&mut artist)?;
                artist
            }
        }
    };
    song.artist_id = artist.id;

    //2. is an album supplied?
    if !raw.album.is_empty() {
        song.album_id = {
            let album_option = db.find_unique(FindAlbum::new(raw.album.clone(), &artist))?;
            match album_option {
                Some(a) => Some(a.id),
                None => {
                    let mut album = Album::new(raw.album.clone(), "".to_string(), artist)?;
                    db.save(&mut album)?;
                    Some(album.id)
                }
            }
        };
    }

    //3. finally create the song
    song.name = raw.title.clone();
    song.spot_id = "".to_string();

    db.save(&mut song)?;

    Ok(song)
}

pub trait SongFav {
    fn fav_song(&self, song: &mut Song, meta: &SongMetadata) -> Result<SongState>;
    fn unfav_song(&self, song: &mut Song, meta: &SongMetadata) -> Result<SongState>;
}

impl<DB> SongFav for DB
    where DB: Save<Song>
{
    fn fav_song(&self, song: &mut Song, meta: &SongMetadata) -> Result<SongState> {
        match song.is_faved {
            false => {
                song.is_faved = true;
                self.save(song)?;
                Ok(SongState::NowFaved)
            }
            true => Ok(SongState::Faved),
        }
    }

    fn unfav_song(&self, song: &mut Song, meta: &SongMetadata) -> Result<SongState> {
        match song.is_faved {
            false => { Ok(SongState::NotFaved) }
            true => {
                song.is_faved = false;
                self.save(song)?;

                Ok(SongState::NowNotFaved)
            }
        }
    }
}

impl Load<Song> for DbPool {
    fn load(&self, id: u64) -> Result<Song> {
        if id == 0 {
            Err(DbError::new("Invalid ID given!"))
        }else {
            match self.get() {
                Ok(mut conn) => {
                    let mut prep_stmt = conn.prepare("SELECT song_title,song_spot_id,is_faved,artist_id,album_id FROM songs WHERE song_id = ? LIMIT 1")?;
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
                        None => Err(DbError::new("Didn't find the song for the given song_id!"))
                    }
                },
                Err(_) => Err(DbError::pool_timeout())
            }
        }
    }
}

impl FollowForeignReference<Song, Album> for DbPool {
    fn follow_reference(&self, to_follow: &Song) -> Result<Album> {
        if to_follow.album_id.is_some() {
            self.load(to_follow.album_id.unwrap())
        } else {
            Err(DbError::new("Song isn't assigned to an album!"))
        }
    }
}

impl FollowForeignReference<Song, Artist> for DbPool {
    fn follow_reference(&self, to_follow: &Song) -> Result<Artist> {
        self.load(to_follow.artist_id)
    }
}

pub struct FindSong(String, u64, Option<u64>);
impl FindSong {
    pub fn new(name: String, artist: &Artist, album: Option<&Album>) -> Self {
        FindSong(name, artist.id, match album{
            Some(a) => Some(a.id),
            None => None
        })
    }
}
impl FindUnique<Song, FindSong> for DbPool {
    fn find_unique(&self, query: FindSong) -> Result<Option<Song>> {
        assert_ne!(query.1, 0);
        assert!(query.2.is_some() && query.2.unwrap() != 0 || query.2.is_none());

        match self.get() {
            Ok(mut conn) => {
                let mut q = "SELECT * FROM songs WHERE song_title = ? AND artist_id = ? AND album_id ".to_string();
                q += if query.2.is_some() { "= ?" } else { "IS ?"};
                q += " LIMIT 1";

                let mut stmt = conn.prepare(&q)?;
                let mut rows = stmt.query(rusqlite::params![query.0, query.1, query.2])?;

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
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl FindUnique<Song, RawSong> for DbPool {
    fn find_unique(&self, query: RawSong) -> Result<Option<Song>> {
        match self.get() {
            Ok(mut conn) => {
                let song_id: u64 = {
                    let mut stmt: rusqlite::Statement;
                    let mut result: rusqlite::Rows;

                    if !query.album.is_empty() {
                        stmt = conn.prepare("SELECT song_id FROM songs,artists,albums WHERE songs.artist_id = artists.artist_id AND songs.album_id = albums.album_id \
                AND songs.song_title = ? AND artists.artist_name = ? AND albums.album_name = ? LIMIT 1")?;
                        result = stmt.query(rusqlite::params![query.title, query.artist, query.album])?;
                    } else {
                        stmt = conn.prepare("SELECT song_id FROM songs,artists WHERE songs.artist_id = artists.artist_id \
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
            },
            Err(_) => Err(DbError::pool_timeout())
        }

    }
}

impl Save<Song> for DbPool {
    fn save(&self, to_save: &mut Song) -> Result<()> {
        debug_assert!(to_save.artist_id != 0);
        match self.get() {
            Ok(mut conn) => {
                if to_save.id == 0 {
                    //Do isert
                    let result : usize = {
                        let mut stmt = conn.prepare("INSERT INTO songs (song_title, song_spot_id, artist_id, album_id, is_faved) VALUES(?,?,?,?,?)")?;
                        stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id, to_save.album_id, to_save.is_faved])?
                    };
                    if result == 1 {
                        to_save.id = last_row_id(&mut conn)?;
                        Ok(())
                    } else {
                        Err(DbError::new("Failed to create new song with the given data!"))
                    }
                } else {
                    //Do update
                    let mut stmt = conn.prepare("UPDATE songs SET song_title = ?, song_spot_id = ?, artist_id = ?, album_id = ?, is_faved = ? WHERE song_id = ?")?;
                    let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id, to_save.album_id, to_save.is_faved, to_save.id])?;

                    if result != 1 {
                        Err(DbError::new("Failed to update the given song!"))
                    }else{
                        Ok(())
                    }
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl Delete<Song> for DbPool {
    fn delete(&self, to_delete: &Song) -> Result<()> {
        match self.get() {
            Ok(mut conn) => {
                delete(&mut conn, "songs", "song_id", to_delete.id)
            },
            Err(_) => Err(DbError::pool_timeout())
        }

    }
}