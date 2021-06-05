use super::{Result, DB, Load, Query, QueryBounds, QueryOrdering, OrderDirection, FindUnique, Save, Delete, FollowForeignReference, db_error::DbError};
use super::util::{last_row_id, delete};
use super::album::{Album, FindAlbum};

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

pub trait AlbumsOfTheWeek {
    fn get_current_album_of_week(&mut self) -> Result<Option<AlbumOfTheWeek>>;
    fn get_albums_of_week(&mut self, offset: u8, limit: u8) -> Result<Vec<AlbumOfTheWeek>>;
    fn store_new_album_of_week(&mut self, album: &mut AlbumOfTheWeek) -> Result<()>;
}

impl AlbumsOfTheWeek for DB {
    fn get_current_album_of_week(&mut self) -> Result<Option<AlbumOfTheWeek>> {
        let mut albums = self.query(
            QueryBounds { offset: 0, page_size: 1 },
            None,
            Some(QueryOrdering{
                direction: OrderDirection::Desc,
                on_field: "source_date".to_string()
            })
        )?;
        Ok(albums.pop())
    }

    fn get_albums_of_week(&mut self, offset: u8, limit: u8) -> Result<Vec<AlbumOfTheWeek>> {
        let albums = self.query(
            QueryBounds{
                offset: offset as u64,
                page_size: limit as u16
            },
            None,
            Some(QueryOrdering{
                direction: OrderDirection::Desc,
                on_field: "source_date".to_string()
            })
        )?;
        Ok(albums)
    }

    fn store_new_album_of_week(&mut self, album: &mut AlbumOfTheWeek) -> Result<()> {
        self.save(album)?;
        Ok(())
    }
}

impl Load<AlbumOfTheWeek> for DB {
    fn load(&mut self, id: u64) -> Result<AlbumOfTheWeek> {
        if id == 0 {
            Err(DbError::new("Invaild ID given!"))
        }else {
            let mut stmt = self.prepare("SELECT album_id,album_song_list_raw,source_name,source_comment,source_date FROM albums_of_week WHERE week_id = ? LIMIT 1")?;
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
                None => Err(DbError::new("Didn't find the album of the week for the given week id!"))
            }
        }
    }
}

impl FollowForeignReference<AlbumOfTheWeek, Album> for DB {
    fn follow_reference(&mut self, to_follow: &AlbumOfTheWeek) -> Result<Album> {
        self.load(to_follow.album_id)
    }
}

pub struct AlbumOfTheWeekQuery();
impl Query<AlbumOfTheWeek, AlbumOfTheWeekQuery> for DB {
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
        let mut stmt = self.prepare(query.as_str())?;
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

impl Save<AlbumOfTheWeek> for DB {
    fn save(&mut self, to_save: &mut AlbumOfTheWeek) -> Result<()> {
        debug_assert!(to_save.album_id != 0);
        let date_time = to_save.source_date.to_rfc3339();

        if to_save.id == 0 {
            //do insert
            let result: usize = {
                let mut stmt = self.prepare("INSERT INTO albums_of_week (album_id,album_song_list_raw,source_name,source_comment,source_date) VALUES(?,?,?,?,?)")?;
                stmt.execute(rusqlite::params![to_save.album_id, to_save.album_song_list_raw, to_save.source, to_save.source_comment, date_time])?
            };
            if result == 1 {
                to_save.id = last_row_id(self)?;
                Ok(())
            }else {
                Err(DbError::new("Failed to create new album of week entry with given data!"))
            }
        }else{
            //Do update
            let mut stmt = self.prepare("UPDATE albums_of_week SET album_id = ?, album_song_list_raw = ?, source_name = ?, source_comment = ?, source_date = ? WHERE week_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.album_id, to_save.album_song_list_raw, to_save.source, to_save.source_comment, date_time, to_save.id])?;

            if result != 1 {
                Err(DbError::new("Failed to update the given album of week!"))
            }else {
                Ok(())
            }
        }
    }
}

impl Delete<AlbumOfTheWeek> for DB {
    fn delete(&mut self, to_delete: &AlbumOfTheWeek) -> Result<()> {
        delete(self, "albums_of_week", "week_id", to_delete.id)
    }
}