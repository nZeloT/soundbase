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

use crate::db_new::{DbApi, Result};
use crate::db_new::db_error::DbError;
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
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::insert_into(track_fav_proposals::table)
                    .values(&new_proposal)
                    .get_result(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn load_track_proposals(&self, page : &Page) -> Result<Vec<TrackFavProposal>> {
        match self.0.get() {
            Ok(conn) => {
                let results = track_fav_proposals::table
                    .filter(track_fav_proposals::track_id.is_null())
                    .offset(page.offset())
                    .limit(page.limit())
                    .load::<TrackFavProposal>(&conn);
                match results {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn find_by_source_and_raw_pattern(&self, source: &str, pattern: &str) -> Result<Option<TrackFavProposal>> {
        match self.0.get() {
            Ok(conn) => {
                let result = track_fav_proposals::table
                    .filter(track_fav_proposals::source_name.like(source))
                    .filter(track_fav_proposals::source_prop.like(pattern))
                    .first(&conn)
                    .optional();
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn link_to_track(&self, id: i32, track_id: i32) -> Result<()> {
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::update(
                    track_fav_proposals::table.filter(track_fav_proposals::track_fav_id.eq(id))
                ).set(track_fav_proposals::track_id.eq(track_id))
                    .execute(&conn);
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }


    fn delete_track_proposal(&self, id: i32) -> Result<()> {
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::delete(
                    track_fav_proposals::table
                        .filter(track_fav_proposals::track_fav_id.eq(id))
                ).execute(&conn);
                match result {
                    Ok(s) => {
                        if s == 1 {
                            Err(DbError::new("Proposal not deleted!"))
                        } else {
                            Ok(())
                        }
                    }
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl FindById<TrackFavProposal> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<TrackFavProposal> {
        match self.0.get() {
            Ok(conn) => {
                let result = track_fav_proposals::table
                    .find(id)
                    .first(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}