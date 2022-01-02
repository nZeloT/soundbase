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

use super::{Result, DbPool, Load, FindUnique, Save, Delete, db_error::DbError};
use super::util::{last_row_id, delete};

#[derive(Default, Debug)]
pub struct Artist {
    pub(super) id: u64,
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

impl Load<Artist> for DbPool {
    fn load(&self, id: u64) -> Result<Artist> {
        if id == 0 {
            Err(DbError::new("Invalid ID given!"))
        }else {
            match self.get() {
                Ok(mut conn) => {
                    let mut prep_stmt = conn.prepare("SELECT artist_name,artist_spot_id FROM artists WHERE artist_id = ? LIMIT 1")?;
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
                            Err(DbError::new("Didn't find the artist for the given artist_id!"))
                        }
                    }
                },
                Err(_) => Err(DbError::pool_timeout())
            }
        }
    }
}

pub struct FindArtist(String);
impl FindArtist {
    pub fn new(name: String) -> Self {
        FindArtist(name)
    }
}
impl FindUnique<Artist, FindArtist> for DbPool {
    fn find_unique(&self, query: FindArtist) -> Result<Option<Artist>> {
        match self.get() {
            Ok(mut conn) => {
                let mut stmt = conn.prepare("SELECT * FROM artists WHERE artist_name = ? LIMIT 1")?;
                let mut rows = stmt.query(rusqlite::params![query.0])?;

                match rows.next()? {
                    Some(row) => Ok(Some(Artist { id: row.get(0)?, name: row.get(1)?, spot_id: row.get(2)? })),
                    None => Ok(None)
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }

    }
}

impl Save<Artist> for DbPool {
    fn save(&self, to_save: &mut Artist) -> Result<()> {
        match self.get() {
            Ok(mut conn) => {
                if to_save.id == 0 {
                    //do insert
                    let result : usize = {
                        let mut stmt = conn.prepare("INSERT INTO artists (artist_name,artist_spot_id) VALUES(?,?)")?;
                        stmt.execute(rusqlite::params![to_save.name, to_save.spot_id])?
                    };
                    if result == 1 {
                        //fetch the new artist ID
                        to_save.id = last_row_id(&mut conn)?;
                        Ok(())
                    } else {
                        Err(DbError::new("Failed to create new artist entry!"))
                    }
                } else {
                    //do update
                    let mut stmt = conn.prepare("UPDATE artists SET artist_name = ?, artist_spot_id = ? WHERE artist_id = ?")?;
                    let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.id])?;

                    if result != 1 {
                        Err(DbError::new("Failed to update the given artist!"))
                    }else{
                        Ok(())
                    }
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }

    }
}

impl Delete<Artist> for DbPool {
    fn delete(&self, to_delete: &Artist) -> Result<()> {
        match self.get() {
            Ok(mut conn) =>  {
                delete(&mut conn, "artists", "artist_id", to_delete.id)

            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}