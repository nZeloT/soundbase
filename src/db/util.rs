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

use super::{Result, DbConn, db_error::DbError};

pub fn last_row_id(db: &mut DbConn) -> Result<u64> {
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

pub fn delete(db: &mut DbConn, table: &'static str, id_field: &'static str, id: u64) -> Result<()> {
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

