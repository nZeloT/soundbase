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

use std::collections::{HashMap, HashSet};
use rspotify::model::{ArtistId, FullArtist, FullTrack};
use serde::{Deserialize, Serialize};
use warp::reply::Reply;
use crate::db_new::album::AlbumDb;
use crate::db_new::album_artist::AlbumArtistsDb;
use crate::db_new::artist::ArtistDb;

use crate::db_new::DbApi;
use crate::db_new::models::{Artist, NewAlbum, NewArtist, NewTrack, Track, TrackFavProposal};
use crate::db_new::track::TrackDb;
use crate::db_new::track_artist::TrackArtistsDb;
use crate::db_new::track_fav_proposal::TrackFavProposalDb;
use crate::error::Error;
use crate::model::{AlbumType, RequestPage, ResponsePage, UniversalId};
use crate::{SpotifyApi, WebResult, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchesQuery {
    pub search: Option<String>,
}

#[derive(Serialize, Debug)]
struct TrackFavProposalListResponse {
    pub entries: Vec<TrackFavProposal>,

    #[serde(flatten)]
    pub page: ResponsePage,
}

impl TrackFavProposalListResponse {
    pub fn new(data: Vec<TrackFavProposal>, page: &RequestPage) -> Self {
        let page = ResponsePage::new("/api/v1/track-proposals/", page, data.len() == page.limit() as usize);
        Self {
            entries: data,
            page,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProposalMatch {
    pub id: UniversalId,
    pub proposal_id: i32,
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
    pub confidence: f32,
}

pub async fn load_proposals(db: DbApi, page: RequestPage) -> WebResult<impl Reply> {
    let api: &dyn TrackFavProposalDb = &db;
    let results = api.load_track_proposals(&page);
    match results {
        Ok(data) => {
            Ok(warp::reply::json(&TrackFavProposalListResponse::new(data, &page)))
        }
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn confirm_proposal(proposal_id: i32, uni_track_id: String, db: DbApi, spotify: SpotifyApi) -> WebResult<impl Reply> {
    let api: &dyn TrackFavProposalDb = &db;
    let result = api.find_by_id(proposal_id);
    match result {
        Ok(proposal) => {
            let ret = confirm_match(db, spotify, proposal, UniversalId::from(&*uni_track_id)).await;
            match ret {
                Ok(_) => Ok(format!("")),
                Err(e) => Err(warp::reject::custom(e))
            }
        }
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn discard_proposal(proposal_id: i32, db: DbApi) -> WebResult<impl Reply> {
    let api: &dyn TrackFavProposalDb = &db;
    let proposal = api.find_by_id(proposal_id);
    match proposal {
        Ok(prop) => {
            let ret = discard_proposal_and_unfav_track(&db, prop);
            match ret {
                Ok(_) => Ok(format!("")),
                Err(e) => Err(warp::reject::custom(e))
            }
        }
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_proposal_matches(proposal_id: i32, query: MatchesQuery, db: DbApi, spotify: SpotifyApi) -> WebResult<impl Reply> {
    let api: &dyn TrackFavProposalDb = &db;
    let proposal = api.find_by_id(proposal_id);
    match proposal {
        Ok(prop) => {
            let ret = find_matches(db, spotify, prop, query.search).await;
            match ret {
                Ok(r) => Ok(warp::reply::json(&r)),
                Err(e) => Err(warp::reject::custom(e))
            }
        }
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

async fn find_matches(db: DbApi, spotify: SpotifyApi, proposal: TrackFavProposal, query: Option<String>) -> Result<Vec<ProposalMatch>> {
    let spotify_search_string = match query {
        Some(q) => q,
        None => build_spotify_query(&proposal)
    };

    //TODO also search on DB; but fuzzy search requires some Indexing

    let search_results = spotify.search(&*spotify_search_string, RequestPage::new(0, 10)).await;
    match search_results {
        Ok(candidates) => {
            let mut matches = candidates.iter()
                .map(|candidate| map_track_to_proposal_match(&proposal, candidate))
                .map(|prop_match| try_find_spotify_id_on_db(&db, prop_match))
                .collect::<Vec<ProposalMatch>>();
            matches.sort_unstable_by(|lhs, rhs| lhs.confidence.partial_cmp(&rhs.confidence).unwrap());
            matches.reverse();
            Ok(matches)
        }
        Err(e) => Err(e)
    }
}

async fn confirm_match(db: DbApi, spotify: SpotifyApi, proposal: TrackFavProposal, matched: UniversalId) -> Result<()> {
    let track_id = match matched {
        UniversalId::Spotify(spot_id) => {
            //Not known to DB yet
            let track = insert_track_from_spotify_id(&db, &spotify, &*spot_id).await?;
            let _ = spotify.save_track(&*spot_id).await?;
            track.track_id
        }
        UniversalId::Database(track_id) => track_id
    };

    //link proposal with track
    let _ = db.link_to_track(proposal.track_fav_id, track_id)?;
    let _ = db.set_faved_state(track_id, true)?;
    Ok(())
}

fn discard_proposal_and_unfav_track(db: &DbApi, proposal: TrackFavProposal) -> Result<()> {
    if let Some(track_id) = proposal.track_id {
        db.set_faved_state(track_id, false)?;
    }
    db.delete_track_proposal(proposal.track_fav_id)?;
    Ok(())
}

fn build_spotify_query(proposal: &TrackFavProposal) -> String {
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

fn map_track_to_proposal_match(proposal: &TrackFavProposal, track: &FullTrack) -> ProposalMatch {
    let confidence = calculate_match_confidence(proposal, &track);
    ProposalMatch {
        id: UniversalId::Spotify(track.id.clone().unwrap().to_string()),
        proposal_id: proposal.track_fav_id,
        title: track.name.clone(),
        album: track.album.name.clone(),
        artists: track.artists.iter().map(|a| a.name.clone()).collect::<Vec<String>>(),
        confidence,
    }
}

fn calculate_match_confidence(proposal: &TrackFavProposal, track: &FullTrack) -> f32 {
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
    } else {
        0 as f64
    };

    let total_score = title_score + artist_score + album_score;
    (total_score / feature_count as f64) as f32
}

fn try_find_spotify_id_on_db<DB>(db: &DB, prop_match: ProposalMatch) -> ProposalMatch
    where DB: TrackDb {
    //try to find the spotify id on known tracks, if found replace it with the database id
    let result = db.find_track_by_universal_id(&prop_match.id);
    match result {
        Ok(opt) => {
            match opt {
                Some(track) => ProposalMatch {
                    id: UniversalId::Database(track.track_id),
                    artists: prop_match.artists,
                    album: prop_match.album,
                    title: prop_match.title,
                    proposal_id: prop_match.proposal_id,
                    confidence: prop_match.confidence,
                },
                None => prop_match
            }
        }
        Err(e) => {
            println!("{:?}", e);
            prop_match
        }
    }
}

async fn insert_track_from_spotify_id(db: &DbApi, spotify: &SpotifyApi, spot_id: &str) -> Result<Track> {
    let spotify_track = spotify.get_track(spot_id).await?;
    let spotify_album = spotify.get_album(&spotify_track.album.id.unwrap()).await?;

    //lets first add the album
    let api: &dyn AlbumDb = db;
    let id = UniversalId::Spotify(spotify_album.id.to_string());
    let db_album = match api.find_by_universal_id(&id)? {
        Some(album) => album,
        None => api.new_full_album(NewAlbum {
            name: &*spotify_album.name,
            year: spotify_album.release_date[..4].parse::<i32>()?,
            total_tracks: spotify_album.tracks.total as i32,
            was_aow: Some(false),
            is_faved: Some(false),
            spot_id: Some(spot_id.to_string()),
            album_type: Some(AlbumType::from(spotify_album.album_type).into()),
        })?
    };

    let album_artist_ids = spotify_album.artists
        .iter()
        .map(|simple_artist| simple_artist.id.clone().unwrap())
        .collect::<HashSet<ArtistId>>();


    //now add the track
    let api: &dyn TrackDb = db;
    let id = UniversalId::Spotify(spotify_track.id.clone().unwrap().to_string());
    let db_track = match api.find_track_by_universal_id(&id)? {
        Some(track) => track,
        None => api.new_full_track(NewTrack {
            title: &*spotify_track.name,
            is_faved: false,
            album_id: db_album.album_id,
            track_number: Some(spotify_track.track_number as i32),
            disc_number: Some(spotify_track.disc_number as i32),
            local_file: None,
            duration_ms: spotify_track.duration.as_millis() as i32,
            spot_id: Some(spotify_track.id.unwrap().to_string()),
        })?
    };

    let track_artist_ids = spotify_track.artists
        .iter()
        .map(|simple_artist| simple_artist.id.clone().unwrap())
        .collect::<HashSet<ArtistId>>();

    let mut all_artist_ids = HashSet::new();
    all_artist_ids.extend(album_artist_ids.clone());
    all_artist_ids.extend(track_artist_ids.clone());

    let spotify_artists = spotify.get_artists(&all_artist_ids.into_iter().collect()).await?;
    let mut artist_id_to_db_artist : HashMap<String, Artist> = HashMap::new();
    for spotify_artist in &spotify_artists {
        let artist = find_or_insert_artist(db, spotify_artist)?;
        artist_id_to_db_artist.insert(spotify_artist.id.to_string(), artist);
    }

    //now link the artist with album and track
    let api : &dyn AlbumArtistsDb = db;
    for artist_id in &album_artist_ids {
        let db_artist = artist_id_to_db_artist.get(&*artist_id.to_string()).unwrap();
        let _ = api.new_album_artist_if_missing(db_artist.artist_id, db_album.album_id)?;
    }

    let api : &dyn TrackArtistsDb = db;
    for artist_id in &track_artist_ids {
        let db_artist =artist_id_to_db_artist.get(&*artist_id.to_string()).unwrap();
        let _ = api.new_track_artist_if_missing(db_track.track_id, db_artist.artist_id)?;
    }

    Ok(db_track)
}

fn find_or_insert_artist<DB>(db: &DB, artist: &FullArtist) -> Result<Artist>
    where DB: ArtistDb {
    let id = UniversalId::Spotify(artist.id.to_string());
    let db_artist = match db.find_artist_by_universal_id(&id)? {
        Some(artist) => artist,
        None => db.new_full_artist(NewArtist {
            name: &*artist.name,
            spot_id: Some(artist.id.to_string()),
        })?
    };
    Ok(db_artist)
}