use super::Player;
use crate::playback::PlaybackError;
use async_trait::async_trait;
use futures::Future;
use librespot::core::cache::Cache;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;
use librespot::discovery::Credentials;
use librespot::playback::audio_backend;
use librespot::playback::config::{AudioFormat, Bitrate, NormalisationType, PlayerConfig};
use librespot::playback::player::PlayerEvent;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SpotifyPlayer {
    librespot_player: Arc<RwLock<librespot::playback::player::Player>>,
    has_track_end_cb: bool,
}

impl SpotifyPlayer {
    pub async fn new(username: &str, password: &str, cache_locations: (&str, &str)) -> Self {
        let session_config = SessionConfig::default();
        let player_config = PlayerConfig {
            bitrate: Bitrate::Bitrate320,
            gapless: false,
            normalisation: true,
            normalisation_type: NormalisationType::Auto,
            ..PlayerConfig::default()
        };
        let audio_format = AudioFormat::default();

        let credentials = Credentials::with_password(username, password);
        let backend = audio_backend::find(None).unwrap();
        let cache = Cache::new(Some(cache_locations.0), Some(cache_locations.1), None).unwrap();

        let session = Session::connect(session_config, credentials, Some(cache))
            .await
            .unwrap();

        let (player, _event_stream) =
            librespot::playback::player::Player::new(player_config, session, None, move || {
                backend(None, audio_format)
            });

        Self {
            librespot_player: Arc::new(RwLock::new(player)),
            has_track_end_cb: false,
        }
    }
}

#[async_trait]
impl Player for SpotifyPlayer {
    async fn connect_player_events(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<super::PlayerEvent>,
    ) {
        if self.has_track_end_cb {
            panic!("Can only set the track end callback once!");
        }

        let mut event_stream = self
            .librespot_player
            .read()
            .await
            .get_player_event_channel();
        tokio::spawn(async move {
            while let Some(evt) = event_stream.recv().await {
                log::info!("Librespot Player Event: {:?}", evt);
                match evt {
                    PlayerEvent::Playing {
                        play_request_id: _id,
                        track_id: _track,
                        position_ms: _pos_ms,
                        duration_ms: _dur_ms,
                    } => {
                        log::info!("Forwarding Playing event!");
                        tx.send(super::PlayerEvent::Playing)
                            .expect("Failed to notify PlayerController");
                    }
                    PlayerEvent::Paused {
                        play_request_id: _id,
                        track_id: _track,
                        position_ms: _pos_ms,
                        duration_ms: _dur_ms,
                    } => {
                        log::info!("Forwarding Paused event!");
                        tx.send(super::PlayerEvent::Paused)
                            .expect("Failed to notify PlayerController");
                    }
                    PlayerEvent::Stopped {
                        play_request_id: _id,
                        track_id: _track,
                    } => {
                        log::info!("Forwarding Stopped event!");
                        tx.send(super::PlayerEvent::Stopped)
                            .expect("Failed to notify PlayerController");
                    }
                    PlayerEvent::EndOfTrack {
                        play_request_id: _id,
                        track_id: _track_id,
                    } => {
                        log::info!("Forwarding End of Track event!");
                        tx.send(super::PlayerEvent::EndOfTrack)
                            .expect("Failed to notify PlayerController");
                    }
                    _ => {}
                }
            }
        });

        self.has_track_end_cb = true;
    }

    async fn start(&self, track_ident: &str) -> Result<(), PlaybackError> {
        let result = SpotifyId::from_uri(track_ident);
        match result {
            Ok(id) => {
                self.librespot_player.write().await.load(id, true, 0);
                log::info!("Track loading in Librespot!");
                Ok(())
            }
            Err(e) => Err(PlaybackError::SpotifyIdError(e)),
        }
    }

    async fn resume(&self) -> Result<(), PlaybackError> {
        self.librespot_player.read().await.play();
        log::info!("Started Playback.");
        Ok(())
    }

    async fn pause(&self) -> Result<(), PlaybackError> {
        self.librespot_player.read().await.pause();
        log::info!("Paused Playback!");
        Ok(())
    }

    async fn seek(&self, target_pos_ms: i64) -> Result<(), PlaybackError> {
        self.librespot_player
            .read()
            .await
            .seek(target_pos_ms as u32);
        log::info!("Seeked to target Position");
        Ok(())
    }
}
