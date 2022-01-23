/*
 * Copyright 2022 nzelot<leontsteiner@gmail.com>
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

use diesel::prelude::*;

use crate::db_new::{DbApi, DbError, Result};
use crate::db_new::FindById;
use crate::db_new::models::{NewTrackFavProposal, TrackFavProposal};
use crate::db_new::schema::*;
use crate::model::Page;

pub trait TrackFavProposalDb: FindById<TrackFavProposal> + Sync {
    fn new_track_proposal(&self, new_proposal: NewTrackFavProposal) -> Result<TrackFavProposal>;
    fn load_track_proposals(&self, page : &Page) -> Result<Vec<TrackFavProposal>>;
    fn find_by_source_and_raw_pattern(&self, source: &str, pattern: &str) -> Result<Option<TrackFavProposal>>;
    fn link_to_track(&self, id: i32, track_id: i32) -> Result<()>;
    fn delete_track_proposal(&self, id: i32) -> Result<()>;
}

impl TrackFavProposalDb for DbApi {
    fn new_track_proposal(&self, new_proposal: NewTrackFavProposal) -> Result<TrackFavProposal> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(track_fav_proposals::table)
            .values(&new_proposal)
            .get_result(&conn);
        Ok(result?)
    }

    fn load_track_proposals(&self, page : &Page) -> Result<Vec<TrackFavProposal>> {
        let conn = self.0.get()?;
        let results = track_fav_proposals::table
            .filter(track_fav_proposals::track_id.is_null())
            .offset(page.offset())
            .limit(page.limit())
            .load::<TrackFavProposal>(&conn);
        Ok(results?)
    }

    fn find_by_source_and_raw_pattern(&self, source: &str, pattern: &str) -> Result<Option<TrackFavProposal>> {
        let conn = self.0.get()?;
        let result = track_fav_proposals::table
            .filter(track_fav_proposals::source_name.like(source))
            .filter(track_fav_proposals::source_prop.like(pattern))
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn link_to_track(&self, id: i32, track_id: i32) -> Result<()> {
        let conn = self.0.get()?;
        let updated = diesel::update(
            track_fav_proposals::table.filter(track_fav_proposals::track_fav_id.eq(id))
        ).set(track_fav_proposals::track_id.eq(track_id))
            .execute(&conn)?;

        if updated == 0 { Ok(()) } else{
            Err(DbError::UpdateError(format!("Failed to link track proposal {} to track {}", id, track_id)))
        }
    }


    fn delete_track_proposal(&self, id: i32) -> Result<()> {
        let conn = self.0.get()?;
        let deleted = diesel::delete(
            track_fav_proposals::table
                .filter(track_fav_proposals::track_fav_id.eq(id))
        ).execute(&conn)?;

        if deleted == 0 { Ok(()) } else{
            Err(DbError::DeleteError(format!("Failed to delete proposal {}", id)))
        }
    }
}

impl FindById<TrackFavProposal> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<TrackFavProposal> {
        let conn = self.0.get()?;
        let result = track_fav_proposals::table
            .find(id)
            .first(&conn);
        Ok(result?)
    }
}