use std::sync::Arc;
use itertools::Itertools;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::playback::{LoopingStates, PlaybackController, PlaybackState, PlaybackTrack};

use super::definition::{
    GetQueueRequest,
    PlaybackBlank,
    PlaybackTrackRequest,
    PlaybackLoopStates,
    PlaybackSeekRequest,
    PlaybackSetLoopingRequest,
    PlaybackSetShuffleRequest,
    PlaybackStateResponse,
    RemoveFromQueueRequest,
    SimpleTrack,
    SimpleAlbum,
    SimpleArtist,
    ToQueueRequest
};
use super::definition::playback_controls_server::PlaybackControls;

pub struct PlaybackControlsService {
    pub(crate) playback : Arc<RwLock<PlaybackController>>
}

#[tonic::async_trait]
impl PlaybackControls for PlaybackControlsService {
    ///
    /// Queue Control functions
    ///
    type GetQueueStream = ReceiverStream<Result<SimpleTrack, Status>>;
    async fn get_queue(&self, _request: Request<GetQueueRequest>) -> Result<Response<Self::GetQueueStream>, Status> {
        //TODO use offset and limit
        let tracks = self.playback.read().await.queue().tracks().await.iter()
            .map(|track| SimpleTrack::from(track))
            .collect_vec();
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            for track in &tracks {
                tx.send(Ok(track.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn append_to_queue(&self, request: Request<ToQueueRequest>) -> Result<Response<PlaybackBlank>, Status> {
        let track_id = request.get_ref().track_id;
        log::info!("Trying to add Track {} to queue", track_id);
        match self.playback.read().await.queue().append(track_id).await {
            Ok(_) => Ok(Response::new(PlaybackBlank {})),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn prepend_to_queue(&self, request: Request<ToQueueRequest>) -> Result<Response<PlaybackBlank>, Status> {
        let track_id = request.get_ref().track_id;
        match self.playback.read().await.queue().prepend(track_id).await {
            Ok(_) => Ok(Response::new(PlaybackBlank {})),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn remove_from_queue(&self, request: Request<RemoveFromQueueRequest>) -> Result<Response<PlaybackBlank>, Status> {
        let index = request.get_ref().queue_position;
        match self.playback.read().await.queue().remove(index as usize).await {
            Ok(_) => Ok(Response::new(PlaybackBlank {})),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn clear_queue(&self, _request: Request<PlaybackBlank>) -> Result<Response<PlaybackBlank>, Status> {
        self.playback.read().await.queue().clear().await;
        Ok(Response::new(PlaybackBlank {}))
    }

    ///
    /// Playback Control Functions
    ///

    async fn play_track(&self, request : Request<PlaybackTrackRequest>) -> Result<Response<PlaybackStateResponse>, Status> {
        let track_id = request.get_ref().track_id;
        match self.playback.read().await.play_directly(track_id).await {
            Ok(_) => Ok(Response::new(PlaybackStateResponse::from(&self.playback.read().await.get_state().await))),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn play(&self, _request: Request<PlaybackBlank>) -> Result<Response<PlaybackStateResponse>, Status> {
        let result = {
            self.playback.write().await.start_playback().await
        };
        match result {
            Ok(_) => {
                Ok(Response::new(PlaybackStateResponse::from(&self.playback.read().await.get_state().await)))
            },
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn pause(&self, _request: Request<PlaybackBlank>) -> Result<Response<PlaybackStateResponse>, Status> {
        let result = {
            self.playback.write().await.pause_playback().await
        };
        match result {
            Ok(_) => Ok(Response::new(PlaybackStateResponse::from(&self.playback.read().await.get_state().await))),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn next(&self, _request: Request<PlaybackBlank>) -> Result<Response<PlaybackStateResponse>, Status> {
        match self.playback.read().await.next_track().await {
            Ok(_) => Ok(Response::new(PlaybackStateResponse::from(&self.playback.read().await.get_state().await))),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn previous(&self, _request: Request<PlaybackBlank>) -> Result<Response<PlaybackStateResponse>, Status> {
        unimplemented!()
    }

    async fn seek(&self, _request: Request<PlaybackSeekRequest>) -> Result<Response<PlaybackStateResponse>, Status> {
        unimplemented!()
    }

    async fn set_shuffle(&self, _request: Request<PlaybackSetShuffleRequest>) -> Result<Response<PlaybackBlank>, Status> {
        unimplemented!()
    }

    async fn set_looping(&self, _request: Request<PlaybackSetLoopingRequest>) -> Result<Response<PlaybackStateResponse>, Status> {
        unimplemented!()
    }


    ///
    /// Playback State Function
    ///
    async fn current_state(&self, _request: Request<PlaybackBlank>) -> Result<Response<PlaybackStateResponse>, Status> {
        let state = {
            self.playback.read().await.get_state().await
        };
        Ok(Response::new(PlaybackStateResponse::from(&state)))
    }

    type StateUpdatesStream = ReceiverStream<Result<PlaybackStateResponse, Status>>;
    async fn state_updates(&self, _request : Request<PlaybackBlank>) -> Result<Response<Self::StateUpdatesStream>, Status> {
        use tokio::sync::mpsc::{Receiver, Sender};

        let mut state_rx = self.playback.read().await.state_update_rx();

        let (tx, rx) : (Sender<Result<PlaybackStateResponse, Status>>,Receiver<Result<PlaybackStateResponse, Status>>) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {

            // Async Channel aus Spot Player -> Playback Controller
            // Async Broadcast aus Playback Controller -> an alle bekannten receiver
            // Async Channel pro Broadcast Receiver -> Tonic Response
            log::info!("Connected new State Update Receiver!");
            while state_rx.changed().await.is_ok() {
                let state = state_rx.borrow().clone();
                let resp = PlaybackStateResponse::from(&state);
                log::info!("Sending State Update using Tonic! {:?}", resp);
                tx.send(Ok(resp))
                    .await
                    .expect("Failed to send State Update to tonic client stream!");
            }
            log::info!("State Update Channel was closed!");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

fn map_opt_playback_track(opt_track : &Option<PlaybackTrack>) -> Option<SimpleTrack> {
        opt_track.as_ref().map(SimpleTrack::from)
}

impl From<&PlaybackTrack> for SimpleTrack {
    fn from(pb : &PlaybackTrack) -> Self {
        Self{
            track_id : pb.meta.track_id,
            title : pb.meta.title.clone(),
            is_faved : pb.meta.is_faved,
            duration_ms : pb.meta.duration_ms,
            album : Some(SimpleAlbum {
                album_id : pb.meta.album.album_id,
                name : pb.meta.album.name.clone()
            }),
            artists : pb.meta.artists.iter().map(|a| SimpleArtist{
                artist_id : a.artist_id,
                name : a.name.clone()
            }).collect_vec()
        }
    }
}

impl From<&PlaybackState> for PlaybackStateResponse {
    fn from(state : &PlaybackState) -> Self {
        Self {
            is_playing : state.is_playing,
            has_previous : state.has_previous,
            has_next : state.has_next,
            loop_state : PlaybackLoopStates::from(&state.looping_state) as i32,
            playing_track : map_opt_playback_track(&state.current_track)
        }
    }
}

impl From<&LoopingStates> for PlaybackLoopStates {
    fn from(state : &LoopingStates) -> Self {
        match state {
            LoopingStates::Off => Self::Off,
            LoopingStates::All => Self::All,
            LoopingStates::One => Self::One
        }
    }
}