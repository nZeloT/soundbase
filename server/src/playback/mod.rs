use std::collections::VecDeque;
use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use log::info;
use rspotify::model::ApiError;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::db_new;
use crate::db_new::album::AlbumDb;
use crate::db_new::track::TrackDb;
use crate::db_new::track_artist::TrackArtistsDb;
use crate::db_new::DbApi;
use crate::model::library_models::{SimpleAlbum, SimpleArtist, SimpleTrack};
use crate::playback::local_player::LocalPlayer;
use crate::playback::spotify_player::SpotifyPlayer;

pub mod local_player;
pub mod spotify_player;

//TODO:
//  - How to handle shuffle mode
//      => in queue?
//  - Handle Looping modes in queue
//  - Implement state handling for playback
//  - Add History to go to previous track
//  - Add event handling for spotify player

#[derive(Clone)]
pub struct PlaybackController {
    queue: PlaybackQueue,
    local_player: LocalPlayer,
    spotify_player: SpotifyPlayer,

    state : Arc<RwLock<PlaybackControllerState>>,
}

#[derive(Clone)]
struct PlaybackControllerState {
    active_player : Option<TargetPlayer>,
    current_track : Option<PlaybackTrack>,
    current_state : ControllerStates,
}

impl PlaybackController {
    pub fn new(
        db: DbApi,
        spotify_player: SpotifyPlayer,
        local_player: LocalPlayer,
    ) -> Result<Self, PlaybackError> {

        let s = Self {
            queue: PlaybackQueue {
                db,
                queued_tracks: Arc::new(RwLock::new(VecDeque::new())),
            },

            local_player,
            spotify_player,

            state : Arc::new(RwLock::new(PlaybackControllerState {
                active_player: None,
                current_track: None,
                current_state: ControllerStates::NotPlaying,
            })),

        };
        Ok(s)
    }

    pub async fn init(&mut self) {
        //connect the spotify player events to the corresponding notify handlers
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        self.spotify_player.connect_track_end_notify(tx).await;
        let this = self.clone();
        tokio::spawn(async move {
            while let Some(_) = rx.recv().await {
                this.notify_end_track().await;
            }
        });
    }

    pub fn queue(&self) -> PlaybackQueue {
        self.queue.clone()
    }

    pub async fn start_playback(&self) -> Result<(), PlaybackError> {
        log::info!("Triggering Playback.");
        match self.state.read().await.current_state {
            ControllerStates::NotPlaying => {
                log::info!("Previous Controller State was NotPlaying. => Fetch first song from Queue and start Playing.");
                debug_assert!(self.state.read().await.active_player == None);
                //get first track from queue and play it
                match self.queue.next_track_for_playback().await {
                    Some(track) => {
                        log::info!("Found track {:?} as next track.", track);
                        self.start_track(track).await?;
                        Ok(())
                    }
                    None => Err(PlaybackError::NoTrackInQueue),
                }
            }

            ControllerStates::Playing => {
                log::info!("Previous Controller State was Playing => Do Nothing");
                {
                    let state = self.state.read().await;
                    debug_assert!(state.active_player.is_some());
                    debug_assert!(state.current_track.is_some());
                }
                Ok(())
            }

            ControllerStates::Paused => {
                log::info!("Previous Controller State was Paused => Start Playing.");
                {
                    let state = self.state.read().await;
                    debug_assert!(state.active_player.is_some());
                    debug_assert!(state.current_track.is_some());
                }
                match self.state.read().await.active_player {
                    Some(TargetPlayer::Spotify) => {
                        log::info!("Resuming on Spotify");
                        self.spotify_player.resume().await?;
                    }
                    Some(TargetPlayer::Local) => {
                        log::info!("Resuming locally");
                    }

                    None => {
                        panic!("Encountered invalid state: Paused without active player!");
                    }
                }
                
                {
                    let mut state = self.state.write().await;
                    state.current_state = ControllerStates::Playing;
                }
                Ok(())
            }
        }
    }

    pub async fn pause_playback(&self) -> Result<(), PlaybackError> {
        log::info!("Triggering Pause Playback");
        match self.state.read().await.current_state {
            ControllerStates::NotPlaying => {
                log::info!("Previous Controller State was NotPlaying => Do Nothing");
                {
                    let state = self.state.read().await;
                    debug_assert!(state.active_player.is_none());
                    debug_assert!(state.current_track.is_none());
                }
                Ok(())
            }

            ControllerStates::Playing => {
                log::info!("Previous Controller State was Playing => Pause Playback");
                match self.state.read().await.active_player {
                    Some(TargetPlayer::Spotify) => {
                        log::info!("Pausing Playback on Spotify");
                        self.spotify_player.pause().await?;
                    }
                    Some(TargetPlayer::Local) => {
                        log::info!("Pausing Playback Locally");
                    }

                    None => {
                        panic!("Encountered invalid state: Playing without active player!");
                    }
                }
                {
                    let mut state = self.state.write().await;
                    state.current_state = ControllerStates::Paused;
                }
                Ok(())
            }

            ControllerStates::Paused => {
                log::info!("Previous Controller State was Paused => Do Nothing.");
                {
                    let state = self.state.read().await;
                    debug_assert!(state.active_player.is_some());
                    debug_assert!(state.current_track.is_some());
                }
                Ok(())
            }
        }
    }

    pub fn next_track(&self) -> Result<(), PlaybackError> {
        unimplemented!()
    }

    pub fn previous_track(&self) -> Result<(), PlaybackError> {
        unimplemented!()
    }

    pub fn seek_to(&self, _target_ms: i64) -> Result<(), PlaybackError> {
        unimplemented!()
    }

    pub fn set_shuffling(&self, _target_state: bool) {
        unimplemented!()
    }

    pub fn set_looping(&self, _target_state: LoopingStates) {
        unimplemented!()
    }

    pub async fn get_state(&self) -> PlaybackState {
        PlaybackState {
            has_previous: false,
            has_next: self.queue.has_next().await,
            looping_state: LoopingStates::Off,
            is_playing: self.state.read().await.current_state == ControllerStates::Playing,
            current_track: self.state.read().await.current_track.clone(),
            playback_position_ms: 0,
        }
    }

    async fn notify_end_track(&self) {
        //handle the track end event properly
        //play next track from queue if any
        match self.queue.next_track_for_playback().await {
            Some(track) => {
                match self.start_track(track).await {
                    Ok(_) => {},
                    Err(e) => panic!("Failed to start scheduled track with Error {:?}", e)
                }
            },
            None => {
                //set state to not playing
                {
                    let mut state = self.state.write().await;
                    state.current_track = None;
                    state.active_player = None;
                    state.current_state = ControllerStates::NotPlaying;
                }
            }
        }
    }

    async fn start_track(&self, track : PlaybackTrack) -> Result<(), PlaybackError> {
        match track.player {
            TargetPlayer::Spotify => {
                log::info!("Playing Track on spotify.");
                self.spotify_player.start(&*track.track_ident).await?;
            }
            TargetPlayer::Local => {
                log::info!("Track would be played locally");
            }
        }

        {
            let mut state = self.state.write().await;
            state.active_player = Some(track.player);
            state.current_track = Some(track);
            state.current_state = ControllerStates::Playing;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub has_previous: bool,
    pub has_next: bool,
    pub looping_state: LoopingStates,
    pub playback_position_ms: i64,
    pub current_track: Option<PlaybackTrack>,
}

#[async_trait]
trait Player {
    async fn connect_track_end_notify(&mut self, tx: tokio::sync::mpsc::UnboundedSender<()>);
    async fn start(&self, track_ident: &str) -> Result<(), PlaybackError>;
    async fn resume(&self) -> Result<(), PlaybackError>;
    async fn pause(&self) -> Result<(), PlaybackError>;
    async fn seek(&self, target_pos_ms: i64) -> Result<(), PlaybackError>;
}

#[derive(Clone)]
pub struct PlaybackQueue {
    db: DbApi,
    queued_tracks: Arc<RwLock<VecDeque<PlaybackTrack>>>,
}

impl PlaybackQueue {
    async fn next_track_for_playback(&self) -> Option<PlaybackTrack> {
        self.queued_tracks.write().await.pop_front()
    }

    pub async fn tracks(&self) -> VecDeque<PlaybackTrack> {
        self.queued_tracks.read().await.clone()
    }

    pub async fn append(&mut self, track_id: i32) -> Result<(), PlaybackError> {
        match self.get_track(track_id) {
            Ok(track) => {
                println!("Found track '{:?}'; adding to queue", track);
                self.queued_tracks.write().await.push_back(track);
                println!(
                    "Current Queue size {}",
                    self.queued_tracks.read().await.len()
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn prepend(&mut self, track_id: i32) -> Result<(), PlaybackError> {
        //the queue doesn't hold the currently playing track
        match self.get_track(track_id) {
            Ok(track) => {
                self.queued_tracks.write().await.push_front(track);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn remove(&mut self, index: usize) -> Result<(), PlaybackError> {
        if index >= self.queued_tracks.read().await.len() {
            return Err(PlaybackError::QueueRemoval);
        }
        let _ = self.queued_tracks.write().await.remove(index);
        Ok(())
    }

    pub async fn clear(&mut self) {
        self.queued_tracks.write().await.clear()
    }

    pub async fn has_next(&self) -> bool {
        !self.queued_tracks.read().await.is_empty()
    }

    fn get_track(&self, track_id: i32) -> Result<PlaybackTrack, PlaybackError> {
        let api: &dyn TrackDb = &self.db;
        match api.find_by_id(track_id)? {
            Some(track) => {
                let artists = self.db.load_artists_for_track(&track)?;
                let album = self.db.load_album_for_track(&track)?;
                let (target_player, track_ident) = if track.local_file.is_some() {
                    (TargetPlayer::Local, track.local_file.unwrap())
                } else {
                    (TargetPlayer::Spotify, track.spot_id.unwrap())
                };

                Ok(PlaybackTrack {
                    meta: SimpleTrack {
                        track_id: track.track_id,
                        title: track.title,
                        is_faved: track.is_faved,
                        duration_ms: track.duration_ms,
                        album: SimpleAlbum {
                            album_id: album.album_id,
                            name: album.name,
                        },
                        artists: artists
                            .iter()
                            .map(|a| SimpleArtist {
                                artist_id: a.artist_id,
                                name: a.name.clone(),
                            })
                            .collect_vec(),
                    },

                    player: target_player,
                    track_ident,
                })
            }
            None => Err(PlaybackError::QueueTrackNotFound),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlaybackTrack {
    pub meta: SimpleTrack,

    player: TargetPlayer,
    track_ident: String,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
enum TargetPlayer {
    Local,
    Spotify,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub enum LoopingStates {
    Off,
    All,
    One,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
enum ControllerStates {
    NotPlaying,
    Playing,
    Paused,
}

#[derive(Error, Debug)]
pub enum PlaybackError {
    #[error("Index larger than queue size!")]
    QueueRemoval,

    #[error("Tried to insert non existent track to queue!")]
    QueueTrackNotFound,

    #[error("DB error occurred: {0}")]
    DbError(#[from] db_new::DbError),

    #[error("Couldn't convert given URI to spotify Id")]
    SpotifyIdError(librespot::core::spotify_id::SpotifyIdError),

    #[error("Can't start playback with no track in queue!")]
    NoTrackInQueue,
}
