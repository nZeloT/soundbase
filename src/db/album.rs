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
use super::artist::Artist;
use rusqlite::ToSql;
use rusqlite::types::ToSqlOutput;

#[derive(Default, Debug)]
pub struct Album {
    pub(super) id: u64,
    pub name: String,
    pub spot_id: String,
    artist_id: u64,
}

impl Album {
    pub fn new(name: String, spot_id: String, artist: Artist) -> Result<Self> {
        if artist.id == 0 {
            return Err(DbError::new("Provided Artist with id 0. This is not allowed! Store Artist first to obtain ID."))
        }
        Ok(Album {
            id: 0,
            name,
            spot_id,
            artist_id: artist.id,
        })
    }
}

impl Load<Album> for DbPool {
    fn load(&self, id: u64) -> Result<Album> {
        if id == 0 {
            Err(DbError::new("Invalid ID given!"))
        }else {
            match self.get() {
                Ok(mut conn) => {
                    let mut prep_stmt = conn.prepare("SELECT album_name, album_spot_id, artist_id FROM albums WHERE album_id = ? LIMIT 1")?;
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
                        None => Err(DbError::new("Didn't find the album for the given album_id!"))
                    }
                },
                Err(_) => Err(DbError::pool_timeout())
            }
        }
    }
}

impl FollowForeignReference<Album, Artist> for DbPool {
    fn follow_reference(&self, to_follow: &Album) -> Result<Artist> {
        self.load(to_follow.artist_id)
    }
}

pub struct FindAlbum(String, u64);
impl FindAlbum {
    pub fn new(name: String, artist: &Artist) -> Self {
        FindAlbum(name, artist.id)
    }
}

impl FindUnique<Album, FindAlbum> for DbPool {
    fn find_unique(&self, query: FindAlbum) -> Result<Option<Album>> {
        assert_ne!(query.1, 0);
        match self.get() {
            Ok(mut conn) => {
                let mut stmt = conn.prepare("SELECT * FROM albums WHERE album_name = ? AND artist_id = ? LIMIT 1")?;
                let mut rows = stmt.query(rusqlite::params![&query.0, query.1])?;

                match rows.next()? {
                    Some(row) => Ok(Some(Album {
                        id: row.get(0)?,
                        artist_id: row.get(1)?,
                        name: row.get(2)?,
                        spot_id: row.get(3)?,
                    })),
                    None => Ok(None)
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }

    }
}

impl Save<Album> for DbPool {
    fn save(&self, to_save: &mut Album) -> Result<()> {
        debug_assert!(to_save.artist_id != 0);
        match self.get() {
            Ok(mut conn) => {
                if to_save.id == 0 {
                    //Do insert
                    let result = {
                        let mut stmt = conn.prepare("INSERT INTO albums (album_name,album_spot_id,artist_id) VALUES(?,?,?)")?;
                        stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id])?
                    };
                    if result == 1 {
                        to_save.id = last_row_id(&mut conn)?;
                        Ok(())
                    } else {
                        Err(DbError::new("Failed to create new album with the given data!"))
                    }
                } else {
                    //Do a update
                    let mut stmt = conn.prepare("UPDATE albums SET album_name = ?, album_spot_id = ?, artist_id = ? WHERE album_id = ?")?;
                    let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.artist_id, to_save.id])?;

                    if result != 1 {
                        Err(DbError::new("Failed to update the given album!"))
                    }else{
                        Ok(())
                    }
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }

    }
}

impl Delete<Album> for DbPool {
    fn delete(&self, to_delete: &Album) -> Result<()> {
        match self.get() {
            Ok(mut conn) => {
                delete(&mut conn, "albums", "album_id", to_delete.id)
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl ToSql for Album {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.id as i64))
    }
}