use super::{Result, DB, Load, Query, FindUnique, Save, Delete, db_error::DbError};
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

impl Load<Artist> for DB {
    fn load(&mut self, id: u64) -> Result<Artist> {
        if id == 0 {
            Err(DbError::new("Invalid ID given!"))
        }else {
            let mut prep_stmt = self.prepare("SELECT artist_name,artist_spot_id FROM artists WHERE artist_id = ? LIMIT 1")?;
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
        }
    }
}

pub struct FindArtist(String);
impl FindArtist {
    pub fn new(name: String) -> Self {
        FindArtist(name)
    }
}
impl FindUnique<Artist, FindArtist> for DB {
    fn find_unique(&mut self, query: FindArtist) -> Result<Option<Artist>> {
        let mut stmt = self.prepare("SELECT * FROM artists WHERE artist_name = ? LIMIT 1")?;
        let mut rows = stmt.query(rusqlite::params![query.0])?;

        match rows.next()? {
            Some(row) => Ok(Some(Artist { id: row.get(0)?, name: row.get(1)?, spot_id: row.get(2)? })),
            None => Ok(None)
        }
    }
}

impl Save<Artist> for DB {
    fn save(&mut self, to_save: &mut Artist) -> Result<()> {
        if to_save.id == 0 {
            //do insert
            let result : usize = {
                let mut stmt = self.prepare("INSERT INTO artists (artist_name,artist_spot_id) VALUES(?,?)")?;
                stmt.execute(rusqlite::params![to_save.name, to_save.spot_id])?
            };
            if result == 1 {
                //fetch the new artist ID
                to_save.id = last_row_id(self)?;
                Ok(())
            } else {
                Err(DbError::new("Failed to create new artist entry!"))
            }
        } else {
            //do update
            let mut stmt = self.prepare("UPDATE artists SET artist_name = ?, artist_spot_id = ? WHERE artist_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.name, to_save.spot_id, to_save.id])?;

            if result != 1 {
                Err(DbError::new("Failed to update the given artist!"))
            }else{
                Ok(())
            }
        }
    }
}

impl Delete<Artist> for DB {
    fn delete(&mut self, to_delete: &Artist) -> Result<()> {
        delete(self, "artists", "artist_id", to_delete.id)
    }
}