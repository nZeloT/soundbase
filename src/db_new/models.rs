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

use crate::db_new::schema::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[table_name="genre"]
#[primary_key(genre_id)]
pub struct Genre {
    pub genre_id : i32,
    pub name : String
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name="genre"]
pub struct NewGenre<'a> {
    pub name : &'a str
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[table_name = "artists"]
#[primary_key(artist_id)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Artist {
    pub artist_id : i32,
    pub name : String,
    pub is_faved : bool,
    pub spot_id : Option<String>
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "artists"]
pub struct NewArtist<'a> {
    pub name : &'a str,
    pub spot_id : Option<String>
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, PartialEq, Debug)]
#[belongs_to(Artist)]
#[belongs_to(Genre)]
#[table_name = "artist_genre"]
#[primary_key(id)]
pub struct ArtistGenre {
    pub id : i32,
    pub artist_id : i32,
    pub genre_id : i32
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "artist_genre"]
pub struct NewArtistGenre {
    pub artist_id : i32,
    pub genre_id : i32
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[table_name = "albums"]
#[primary_key(album_id)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Album {
    pub album_id : i32,
    pub name : String,
    pub album_type : i32,
    pub year : i32,
    pub total_tracks : Option<i32>,
    pub is_faved : bool,
    pub was_aow : bool,
    pub spot_id : Option<String>
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "albums"]
pub struct NewAlbum<'a> {
    pub name : &'a str,
    pub album_type : Option<i32>,
    pub year : i32,
    pub total_tracks : i32,
    pub is_faved : Option<bool>,
    pub was_aow : Option<bool>,
    pub spot_id : Option<String>
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[belongs_to(Album)]
#[belongs_to(Artist)]
#[table_name = "album_artists"]
#[primary_key(id)]
pub struct AlbumArtists {
    pub id : i32,
    pub album_id : i32,
    pub artist_id : i32
}

#[derive(Insertable)]
#[table_name = "album_artists"]
pub struct NewAlbumArtists {
    pub album_id : i32,
    pub artist_id : i32
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[belongs_to(Album)]
#[table_name = "albums_of_week"]
#[primary_key(aow_id)]
#[changeset_options(treat_none_as_null = "true")]
pub struct AlbumOfWeek {
    pub aow_id : i32,
    pub album_id : i32,
    pub year : i32,
    pub week : i32,
    pub source_name : String,
    pub source_comment : String,
    pub source_date : String,
    pub track_list_raw : Option<String>
}

#[derive(Insertable)]
#[table_name = "albums_of_week"]
pub struct NewAlbumOfWeek<'a> {
    pub album_id : i32,
    pub year : i32,
    pub week : i32,
    pub source_name : &'a str,
    pub source_comment : &'a str,
    pub source_date : &'a str,
    pub track_list_raw : Option<String>
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[belongs_to(Album)]
#[table_name = "tracks"]
#[primary_key(track_id)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Track {
    pub track_id : i32,
    pub title : String,
    pub album_id : i32,
    pub disc_number : Option<i32>,
    pub track_number : Option<i32>,
    pub duration_ms : i32,
    pub is_faved : bool,
    pub local_file : Option<String>,
    pub spot_id : Option<String>
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[belongs_to(Track)]
#[belongs_to(Artist)]
#[table_name = "track_artist"]
#[primary_key(id)]
pub struct TrackArtists {
    pub id : i32,
    pub track_id : i32,
    pub artist_id : i32
}

#[derive(Insertable)]
#[table_name = "track_artist"]
pub struct NewTrackArtists {
    pub track_id : i32,
    pub artist_id : i32
}

#[derive(Insertable)]
#[table_name = "tracks"]
pub struct NewTrack<'a> {
    pub title : &'a str,
    pub album_id : i32,
    pub disc_number : Option<i32>,
    pub track_number : Option<i32>,
    pub duration_ms : i32,
    pub is_faved : bool,
    pub local_file : Option<String>,
    pub spot_id : Option<String>
}

#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug)]
#[belongs_to(Track)]
#[table_name = "charts_of_week"]
#[primary_key(chart_id)]
pub struct ChartsOfWeekEntry {
    pub chart_id : i32,
    pub year : i32,
    pub calendar_week : i32,
    pub source_name : String,
    pub track_id : i32,
    pub chart_position : i32
}

#[derive(Insertable)]
#[table_name = "charts_of_week"]
pub struct NewChartsOfWeek {
    pub year : i32,
    pub calendar_week : i32,
    pub source_name : String,
    pub track_id : i32,
    pub chart_position : i32
}


#[derive(Queryable, Identifiable, Associations, AsChangeset, PartialEq, Debug, Serialize, Deserialize)]
#[belongs_to(Track)]
#[table_name = "track_fav_proposals"]
#[primary_key(track_fav_id)]
pub struct TrackFavProposal {
    pub track_fav_id : i32,
    pub source_name : String,
    pub source_prop : String,
    pub ext_track_title : String,
    pub ext_artist_name : String,
    pub ext_album_name : Option<String>,
    pub track_id : Option<i32>
}

#[derive(Insertable)]
#[table_name = "track_fav_proposals"]
pub struct NewTrackFavProposal {
    pub source_name : String,
    pub source_prop : String,
    pub ext_track_title : String,
    pub ext_artist_name : String,
    pub ext_album_name : Option<String>
}