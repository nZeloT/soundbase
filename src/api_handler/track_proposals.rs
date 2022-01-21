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

//  1. /api/v1/track-proposal/
//      => gepagtes laden mit ?&offset=&limit?
//  2. /api/v1/track-proposal/<id>/confirm/<track-uuid-id>
//  3. /api/v1/track_proposal/<id>/discard
//  4. /api/v1/track-proposal/<id>/matches?search=

use http::StatusCode;
use rspotify::model::{ArtistId, FullTrack};
use serde::{Deserialize, Serialize};
use warp::reply::Reply;

use crate::db_new::DbApi;
use crate::db_new::models::{Track, TrackFavProposal};
use crate::db_new::track::TrackDb;
use crate::db_new::track_fav_proposal::TrackFavProposalDb;
use crate::error::SoundbaseError;
use crate::model::{Page, UniversalTrackId};
use crate::SpotifyApi;

use super::{handle_error, reply, reply_json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchesQuery {
    pub search: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TrackFavProposalList {
    pub entries: Vec<TrackFavProposal>,

    #[serde(flatten)]
    pub page: Page,
}

impl TrackFavProposalList {
    pub fn new(data: Vec<TrackFavProposal>, page: &Page) -> Self {
        Self {
            entries: data,
            page: Page { offset: Some(page.offset()), limit: Some(page.limit()) },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProposalMatch {
    pub id : UniversalTrackId,
    pub proposal_id : i32,
    pub title : String,
    pub album : String,
    pub artists : Vec<String>,
    pub confidence : f32
}

pub async fn load_proposals(db: DbApi, page: Page) -> Result<warp::reply::Response, std::convert::Infallible> {
    let api: &dyn TrackFavProposalDb = &db;
    let results = api.load_track_proposals(&page);
    match results {
        Ok(data) => {
            let ret = TrackFavProposalList::new(data, &page);
            Ok(reply_json(&ret, StatusCode::OK).into_response())
        }
        Err(e) => Ok(handle_error("Failed to load proposals!", e))
    }
}

pub async fn confirm_proposal(proposal_id: i32, uni_track_id: String, db: DbApi, spotify : SpotifyApi) -> Result<impl warp::Reply, std::convert::Infallible> {
    let api : &dyn TrackFavProposalDb = &db;
    let result = api.find_by_id(proposal_id);
    match result {
        Ok(proposal) => {
            let ret = confirm_match(db, spotify, proposal, UniversalTrackId::from(&*uni_track_id)).await;
            match ret {
                Ok(_) => Ok(reply("", StatusCode::OK).into_response()),
                Err(e) => Ok(handle_error("Failed to confirm proposal", e))
            }
        },
        Err(e) => Ok(handle_error("Failed to find proposal!", e))
    }
}

pub async fn discard_proposal(proposal_id: i32, db: DbApi) -> Result<warp::reply::Response, std::convert::Infallible> {
    let api : &dyn TrackFavProposalDb = &db;
    let proposal = api.find_by_id(proposal_id);
    match proposal {
        Ok(prop) => {
            let ret = discard_proposal_and_unfav_track(&db, prop);
            match ret {
                Ok(_) => Ok(reply("", StatusCode::OK).into_response()),
                Err(e) => Ok(handle_error("Failed to discard matches!", e))
            }
        },
        Err(e) => Ok(handle_error("Failed to find proposal!", e))
    }
}

pub async fn load_proposal_matches(proposal_id: i32, query: MatchesQuery, db: DbApi, spotify : SpotifyApi) -> Result<warp::reply::Response, std::convert::Infallible> {
    let api : &dyn TrackFavProposalDb = &db;
    let proposal = api.find_by_id(proposal_id);
    match proposal {
        Ok(prop) => {
            let ret = find_matches(db, spotify, prop, query.search).await;
            match ret {
                Ok(r) => Ok(reply_json(&r, StatusCode::OK).into_response()),
                Err(e) => Ok(handle_error("Failed to calculate matches!", e))
            }
        },
        Err(e) => Ok(handle_error("Failed to find proposal!", e))
    }
}

async fn find_matches(db : DbApi, spotify : SpotifyApi, proposal : TrackFavProposal, query : Option<String>) -> Result<Vec<ProposalMatch>, SoundbaseError> {
    let spotify_search_string = match query {
        Some(q) => q,
        None => build_spotify_query(&proposal)
    };

    //TODO also search on DB; but fuzzy search requires some Indexing

    let search_results = spotify.search(&*spotify_search_string, Page::new(0, 10)).await;
    match search_results {
        Ok(candidates) => {
            let matches = candidates.iter()
                .map(|candidate| map_track_to_proposal_match(&proposal, candidate))
                .map(|prop_match|try_find_spotify_id_on_db(&db, prop_match))
                .collect::<Vec<ProposalMatch>>();

            Ok(matches)
        },
        Err(e) => Err(e)
    }
}

async fn confirm_match(db : DbApi, spotify : SpotifyApi, proposal : TrackFavProposal, matched : UniversalTrackId) -> Result<(), SoundbaseError> {
    let track_id = match matched {
        UniversalTrackId::Spotify(spot_id) => {
            //Not known to DB yet
            let track = insert_track_from_spotify_id(&db, &spotify, &*spot_id).await?;
            track.track_id
        },
        UniversalTrackId::Database(track_id) => track_id
    };

    //link proposal with track
    let _ = db.link_to_track(proposal.track_fav_id, track_id)?;
    let _ = db.set_faved_state(track_id, true)?;
    Ok(())
}

fn discard_proposal_and_unfav_track(db : &DbApi, proposal : TrackFavProposal) -> Result<(), SoundbaseError> {
    if let Some(track_id) = proposal.track_id {
        db.set_faved_state(track_id, false)?;
    }
    db.delete_track_proposal(proposal.track_fav_id)?;
    Ok(())
}

fn build_spotify_query(proposal : &TrackFavProposal) -> String {
    let mut search = "track:".to_string();
    search += &*proposal.ext_track_title;
    search += " artist:";
    search += &*proposal.ext_artist_name;
    if let Some(ext_album) = &proposal.ext_album_name {
        search += " album:";
        search += ext_album;
    }
    search
}

fn map_track_to_proposal_match(proposal: &TrackFavProposal, track : &FullTrack) -> ProposalMatch {
    let confidence = calculate_match_confidence(proposal, &track);
    ProposalMatch{
        id: UniversalTrackId::Spotify(track.id.unwrap().to_string()),
        proposal_id: proposal.track_fav_id,
        title: track.name.clone(),
        album: track.album.name.clone(),
        artists: track.artists.iter().map(|a| a.name).collect::<Vec<String>>(),
        confidence
    }
}

fn calculate_match_confidence(proposal : &TrackFavProposal, track : &FullTrack) -> f32 {

    let track_title = &*track.name;
    let prop_title = &*proposal.ext_track_title;
    let title_score = strsim::normalized_levenshtein(prop_title, track_title);

    //TODO: also match on multiple artists
    let track_artist = &*track.artists[0].name;
    let prop_artist = &*proposal.ext_artist_name;
    let artist_score = strsim::normalized_levenshtein(prop_artist, track_artist);

    let mut feature_count = 2;

    let album_score = if let Some(prop_album) = &proposal.ext_album_name {
        feature_count += 1;
        let track_album = &*track.album.name;
        strsim::normalized_levenshtein(prop_album, track_album)
    }else{
        0 as f64
    };

    let total_score = title_score + artist_score + album_score;
    (total_score / feature_count as f64) as f32
}

fn try_find_spotify_id_on_db<DB>(db : &DB, prop_match : ProposalMatch) -> ProposalMatch
where DB : TrackDb {
    //try to find the spotify id on known tracks, if found replace it with the database id
    let result = db.find_track_by_universal_id(&prop_match.id);
    match result {
        Ok(opt) => {
            match opt {
                Some(track) => ProposalMatch{
                    id: UniversalTrackId::Database(track.track_id),
                    artists: prop_match.artists,
                    album: prop_match.album,
                    title: prop_match.title,
                    proposal_id: prop_match.proposal_id,
                    confidence: prop_match.confidence
                },
                None => prop_match
            }
        },
        Err(e) => {
            println!("{:?}", e);
            prop_match
        }
    }
}

async fn insert_track_from_spotify_id(db : &DbApi, spotify : &SpotifyApi, spot_id : &str) -> Result<Track, SoundbaseError> {
    let spotify_track = spotify.get_track(spot_id).await?;
    let spotify_album = spotify.get_album(&spotify_track.album.id.unwrap()).await?;
    let artist_ids = spotify_track.artists
        .iter()
        .map(|simple_artist| simple_artist.id.clone().unwrap())
        .collect::<Vec<ArtistId>>();
    let spotify_artists = spotify.get_artists(&artist_ids).await?;



    todo!()
}

