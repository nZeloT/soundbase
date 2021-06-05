use super::{Result, DB, db_error::DbError};

pub fn last_row_id(db: &mut DB) -> Result<u64> {
    let mut prep_stmt = db.prepare("SELECT last_insert_rowid()")?;
    let mut rows = prep_stmt.query(rusqlite::params![])?;
    match rows.next()? {
        Some(row) => {
            let id : u64 = row.get(0)?;
            Ok(id)
        },
        None => Err(DbError::new("Failed to receive new last insert rowid!"))
    }
}

pub fn delete(db: &mut DB, table: &'static str, id_field: &'static str, id: u64) -> Result<()> {
    if id == 0 {
        Err(DbError::new("Can't delete row with ID 0!"))
    }else {
        let mut prep_stmt = db.prepare(("DELETE FROM ".to_owned() + table + " WHERE " + id_field + " = ?").as_str())?;
        let result = prep_stmt.execute([id])?;

        if result != 1 {
            Err(DbError::new("Failed to delete row from table"))
        }else{
            Ok(())
        }
    }
}

