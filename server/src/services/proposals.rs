use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use crate::db_new::album_artist::AlbumArtistsDb;
use super::definition::track_favourites_server::TrackFavourites;
use super::definition::{
    NewTrackFavouriteRequest,
    NewTrackFavouritesResponse,
    NewTrackFavouritesState,
    ExistsTrackFavouriteRequest,
    ExistsTrackFavouriteState,
    ListTrackFavouritesRequest,
    ListTrackFavouritesMatchesRequest,
    ConfirmTrackFavouriteRequest,
    DiscardTrackFavouritesRequest,
    ExternalDiscardTrackFavouritesRequest,
    TrackFavourite,
    TrackFavouriteMatch,
    ExistsTrackFavouriteResponse,
    TrackFavouritesBlank
};

use crate::db_new::DbApi;
use crate::db_new::models::{Artist, NewTrackFavProposal, Track, TrackFavProposal};
use crate::db_new::track::TrackDb;
use crate::db_new::track_artist::TrackArtistsDb;
use crate::db_new::track_fav_proposal::TrackFavProposalDb;
use crate::model::{RequestPage, UniversalId};
use crate::spotify::db_utils::{get_or_create_album, get_or_create_artist, get_or_create_track};
use crate::SpotifyApi;

pub struct TrackProposalsService {
    pub(crate) db : DbApi,
    pub(crate) spotify : SpotifyApi
}

trait RawResolver {
    fn is_excluded(&self, raw: &str) -> bool;
    fn track(&self, raw: &str) -> String;
    fn album(&self, raw: &str) -> Option<String>;
    fn artist(&self, raw: &str) -> String;
}

#[tonic::async_trait]
impl TrackFavourites for TrackProposalsService {
    async fn new_proposal(&self, request : Request<NewTrackFavouriteRequest>)
        -> Result<Response<NewTrackFavouritesResponse>, Status> {
        let req = request.get_ref();
        //let source_kind : String = req.source_kind;
        let source_name : String = req.source_name.clone();
        let source_raw : String  = req.source_raw.clone();

        let resolver = get_resolver(&*source_name);
        let resp : NewTrackFavouritesState = if resolver.is_excluded(&*source_raw) {
            NewTrackFavouritesState::Excluded
        } else {
            let proposal = find_on_db(&self.db, &*source_name, &*source_raw)?;
            match proposal {
                Some(_) => NewTrackFavouritesState::Exists,
                None => {
                    let _ = self.db.new_track_proposal(
                        get_new_track_proposal(&resolver, &*source_name, &*source_raw))?;
                    NewTrackFavouritesState::Created
                }
            }
        };

        Ok(Response::new(NewTrackFavouritesResponse{
            state : resp as i32
        }))
    }

    async fn exists(&self, request : Request<ExistsTrackFavouriteRequest>)
        -> Result<Response<ExistsTrackFavouriteResponse>, Status> {
        let req = request.get_ref();
        //let source_kind : String = req.source_kind;
        let source_name : String = req.source_name.clone();
        let source_raw : String  = req.source_raw.clone();

        let resolver = get_resolver(&*source_name);
        let resp : ExistsTrackFavouriteState = if resolver.is_excluded(&*source_raw) {
            ExistsTrackFavouriteState::Excluded
        } else {
            let proposal = find_on_db(&self.db, &*source_name, &*source_raw)?;
            match proposal {
                Some(_) => ExistsTrackFavouriteState::Found,
                None => ExistsTrackFavouriteState::NotFound
            }
        };

        Ok(Response::new(ExistsTrackFavouriteResponse {
            state : resp as i32
        }))
    }

    type ListStream = ReceiverStream<Result<TrackFavourite, Status>>;
    async fn list(&self, request : Request<ListTrackFavouritesRequest>)
        -> Result<Response<Self::ListStream>, Status> {
        let offset : i32 = request.get_ref().offset;
        let limit  : i32  = request.get_ref().limit;

        let api : &dyn TrackFavProposalDb = &self.db;
        let proposals = api
            .load_track_proposals(&RequestPage::new(offset as i64, limit as i64))?
            .iter()
            .map(|p|TrackFavourite{
                id : p.track_fav_id,
                opt_track_id : if let Some(tid) = p.track_id { tid } else { -1 },
                source_name : p.source_name.clone(),
                source_prop : p.source_prop.clone(),
                ext_track_title : p.ext_track_title.clone(),
                ext_artist_name : p.ext_artist_name.clone(),
                ext_album_name : if let Some(a) = &p.ext_album_name { a.clone() } else { "".to_string() }
            }).collect_vec();
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            for p in &proposals {
                tx.send(Ok(p.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type MatchesStream = ReceiverStream<Result<TrackFavouriteMatch, Status>>;
    async fn matches(&self, request : Request<ListTrackFavouritesMatchesRequest>)
        -> Result<Response<Self::MatchesStream>, Status> {
        let proposal_id : i32 = request.get_ref().track_fav_id;
        let opt_search : String = request.get_ref().opt_search.clone();
        let api : &dyn TrackFavProposalDb = &self.db;
        match api.find_by_id(proposal_id)? {
            Some(proposal) => {
                let opt_search = if opt_search.is_empty() { None } else { Some(opt_search) };
                let result = find_matches(&self.db, &self.spotify, proposal, opt_search).await;
                if let Err(e) = result {
                    return Err(Status::internal(e.to_string()));
                }
                let matches = result.unwrap();
                let (tx, rx) = tokio::sync::mpsc::channel(10);
                tokio::spawn(async move {
                    for m in &matches {
                        tx.send(Ok(m.clone())).await.unwrap();
                    }
                });
                Ok(Response::new(ReceiverStream::new(rx)))
            },
            None => Err(Status::not_found("No Proposal found for given id!"))
        }
    }

    async fn confirm(&self, request : Request<ConfirmTrackFavouriteRequest>)
        -> Result<Response<TrackFavouritesBlank>, Status> {
        let proposal_id = request.get_ref().track_favourite_id;
        let match_id    = request.get_ref().match_id.clone();
        let uni_id      = UniversalId::from(&*match_id);
        let api : &dyn TrackFavProposalDb = &self.db;
        let result = api.find_by_id(proposal_id);
        match result {
            Ok(opt) => {
                match opt {
                    Some(proposal) => {
                        let ret = confirm_match(&self.db, &self.spotify, proposal, uni_id).await;
                        match ret {
                            Ok(_) => Ok(Response::new(TrackFavouritesBlank{})),
                            Err(e) => Err(Status::internal(e.to_string()))
                        }
                    },
                    None => Err(Status::internal("Proposal not found!"))
                }
            },
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn discard(&self, request : Request<DiscardTrackFavouritesRequest>)
        -> Result<Response<TrackFavouritesBlank>, Status> {
        let proposal_id = request.get_ref().track_favourites_id;
        let api : &dyn TrackFavProposalDb = &self.db;
        match api.find_by_id(proposal_id) {
            Ok(opt) => {
                match opt {
                    Some(proposal) => {
                        let ret = discard_proposal_and_unfav_track(&self.db, proposal);
                        match ret {
                            Ok(_) => Ok(Response::new(TrackFavouritesBlank{})),
                            Err(e) => Err(Status::internal(e.to_string()))
                        }
                    },
                    None => Err(Status::internal("Proposal not found!"))
                }
            },
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn external_discard(&self, request : Request<ExternalDiscardTrackFavouritesRequest>)
        -> Result<Response<TrackFavouritesBlank>, Status> {
        let req = request.get_ref();
        //let source_kind : String = req.source_kind;
        let source_name : String = req.source_name.clone();
        let source_raw : String  = req.source_raw.clone();

        let resolver = get_resolver(&*source_name);
        if !resolver.is_excluded(&*source_raw) {
            let proposal = find_on_db(&self.db, &*source_name, &*source_raw)?;
            if let Some(p) = proposal {
                set_track_to_unfaved(&self.db, &p.track_id)?;
                self.db.delete_track_proposal(p.track_fav_id)?;
            }
        }

        Ok(Response::new(TrackFavouritesBlank {}))
    }
}

fn get_resolver(source_name: &str) -> Box<dyn RawResolver> {
    match source_name {
        "Rock Antenne" => Box::new(RockAntenneResolver()),
        _ => Box::new(FullResolver())
    }
}

fn find_on_db<DB>(db: &DB, source_name : &str, pattern : &str) -> Result<Option<TrackFavProposal>, crate::db_new::DbError>
    where DB: TrackFavProposalDb {
    db.find_by_source_and_raw_pattern(source_name, pattern)
}

fn get_new_track_proposal(resolver : &Box<dyn RawResolver>, source_name : &str, raw : &str) -> NewTrackFavProposal {
    let ext_artist = resolver.artist(raw);
    let ext_track = resolver.track(raw);
    let ext_album = resolver.album(raw);

    NewTrackFavProposal {
        source_name: source_name.to_string(),
        source_prop: raw.to_string(),
        ext_track_title: ext_track,
        ext_artist_name: ext_artist,
        ext_album_name: ext_album,
    }
}

fn set_track_to_unfaved<DB>(db: &DB, track_id: &Option<i32>) -> Result<(), crate::db_new::DbError>
    where DB: TrackDb {
    match track_id {
        Some(id) => {
            db.set_faved_state(*id, false)?;
            Ok(())
        }
        None => Ok(())
    }
}

async fn find_matches(db: &DbApi, spotify: &SpotifyApi, proposal: TrackFavProposal, query: Option<String>) -> Result<Vec<TrackFavouriteMatch>, Box<dyn std::error::Error>> {
    let spotify_search_string = match query {
        Some(q) => q,
        None => build_spotify_query(&proposal)
    };

    //TODO also search on DB; but fuzzy search requires some Indexing

    let candidates = spotify.search(&*spotify_search_string, RequestPage::new(0, 5)).await?;
    let mut unlinked : Vec<rspotify::model::FullTrack> = Vec::new();
    for candidate in candidates {
        let track = match candidate.linked_from {
            Some(link) => spotify.get_track(&link.id).await?,
            None => candidate
        };
        unlinked.push(track);
    }

    let mut matches = unlinked.iter()
        .unique_by(|&track| track.id.as_ref().unwrap().clone())
        .map(|candidate| map_track_to_proposal_match(&proposal, candidate))
        .map(|prop_match| try_find_spotify_id_on_db(db, prop_match))
        .collect::<Vec<TrackFavouriteMatch>>();

    matches.sort_by(|lhs, rhs| {
        let cmp = lhs.confidence.partial_cmp(&rhs.confidence).unwrap();
        if cmp == Ordering::Equal {
            match lhs.album_year.partial_cmp(&rhs.album_year).unwrap() {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            }
        }else {
            cmp
        }
    });
    matches.reverse();
    Ok(matches)
}

async fn confirm_match(db: &DbApi, spotify: &SpotifyApi, proposal: TrackFavProposal, matched: UniversalId) -> Result<(), Box<dyn std::error::Error>> {
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
    let api : &dyn TrackDb = db;
    let _ = api.set_faved_state(track_id, true)?;
    Ok(())
}

fn discard_proposal_and_unfav_track(db: &DbApi, proposal: TrackFavProposal) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(track_id) = proposal.track_id {
        let api : &dyn TrackDb = db;
        api.set_faved_state(track_id, false)?;
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

fn map_track_to_proposal_match(proposal: &TrackFavProposal, track: &rspotify::model::FullTrack) -> TrackFavouriteMatch {
    let confidence = calculate_match_confidence(proposal, &track);
    TrackFavouriteMatch {
        match_id: track.id.clone().unwrap().to_string(),
        track_favourite_id: proposal.track_fav_id,
        title: track.name.clone(),
        album: track.album.name.clone(),
        album_year: track.album.release_date.as_ref().unwrap()[..4].parse::<i32>().unwrap(),
        artists: track.artists.iter().map(|a| a.name.clone()).collect::<Vec<String>>(),
        confidence,
    }
}

fn calculate_match_confidence(proposal: &TrackFavProposal, track: &rspotify::model::FullTrack) -> f32 {
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

fn try_find_spotify_id_on_db<DB>(db: &DB, prop_match: TrackFavouriteMatch) -> TrackFavouriteMatch
    where DB: TrackDb {
    //try to find the spotify id on known tracks, if found replace it with the database id
    let result = db.find_track_by_universal_id(&UniversalId::from(&*prop_match.match_id));
    match result {
        Ok(opt) => {
            match opt {
                Some(track) => TrackFavouriteMatch {
                    match_id: track.track_id.to_string(),
                    artists: prop_match.artists,
                    album: prop_match.album,
                    album_year: prop_match.album_year,
                    title: prop_match.title,
                    track_favourite_id: prop_match.track_favourite_id,
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

async fn insert_track_from_spotify_id(db: &DbApi, spotify: &SpotifyApi, spot_id: &str) -> Result<Track, Box<dyn std::error::Error>> {
    let spotify_track = spotify.get_track_from(spot_id).await?;
    let spotify_album = spotify.get_album(&spotify_track.album.id.as_ref().clone().unwrap()).await?;

    //lets first add the album
    let db_album = get_or_create_album(db, &spotify_album)?;

    let album_artist_ids = spotify_album.artists
        .iter()
        .map(|simple_artist| simple_artist.id.clone().unwrap())
        .collect::<HashSet<rspotify::model::ArtistId>>();


    //now add the track
    let db_track = get_or_create_track(db, &db_album, &spotify_track)?;

    let track_artist_ids = spotify_track.artists
        .iter()
        .map(|simple_artist| simple_artist.id.clone().unwrap())
        .collect::<HashSet<rspotify::model::ArtistId>>();

    let mut all_artist_ids = HashSet::new();
    all_artist_ids.extend(album_artist_ids.clone());
    all_artist_ids.extend(track_artist_ids.clone());

    let spotify_artists = spotify.get_artists(&all_artist_ids.into_iter().collect::<Vec<rspotify::model::ArtistId>>()).await?;
    let mut artist_id_to_db_artist : HashMap<String, Artist> = HashMap::new();
    for spotify_artist in &spotify_artists {
        let artist = get_or_create_artist(db, spotify_artist)?;
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

struct RockAntenneResolver();

struct FullResolver();

impl RawResolver for RockAntenneResolver {
    fn is_excluded(&self, raw: &str) -> bool {
        raw.starts_with("ROCK ANTENNE")
    }

    fn track(&self, raw: &str) -> String {
        raw.split('-').collect::<Vec<&str>>()[1].trim().to_string()
    }

    fn album(&self, _: &str) -> Option<String> {
        None
    }

    fn artist(&self, raw: &str) -> String {
        raw.split('-').collect::<Vec<&str>>()[0].trim().to_string()
    }
}

impl RawResolver for FullResolver {
    fn is_excluded(&self, _: &str) -> bool {
        false
    }

    //format is "Title - Arist (Album)"
    fn track(&self, raw: &str) -> String {
        raw.split('-').collect::<Vec<&str>>()[0].trim().to_string()
    }

    fn album(&self, raw: &str) -> Option<String> {
        let artist_album = raw.split('-').collect::<Vec<&str>>()[1].trim().to_string();
        let album = artist_album.split('(').collect::<Vec<&str>>()[1].trim().to_string();
        Some(album[..album.len() - 1].to_string())
    }

    fn artist(&self, raw: &str) -> String {
        let artist_album = raw.split('-').collect::<Vec<&str>>()[1].trim().to_string();
        artist_album.split('(').collect::<Vec<&str>>()[0].trim().to_string()
    }
}