use adw::glib::Sender;
use async_trait::async_trait;
use tonic::codegen::StdError;
use tonic::transport::{Endpoint, Error};
use library_api::{LibraryEntity, LibraryRequests, LibraryResponse};
use crate::api::api_base::{ApiBase, ApiClientBase, ApiTypes};

pub mod services {
    tonic::include_proto!("soundbase");
}

mod api_base;
mod library_api;
mod playback_api;

type LibraryClient = services::library_client::LibraryClient<tonic::transport::Channel>;
type PlaybackClient = services::playback_controls_client::PlaybackControlsClient<tonic::transport::Channel>;

pub use api_base::ApiRuntime;
use crate::api::playback_api::{PlaybackRequest, PlaybackResponse};

///
/// Library API
///
#[derive(Clone, Debug)]
pub struct LibraryApi(ApiBase<Self>);

#[async_trait]
impl ApiTypes for LibraryApi {
    type Request = LibraryRequests;
    type Response = LibraryResponse;
    type Client = LibraryClient;

    async fn process_request(glib_sender : gtk4::glib::Sender<Result<Self::Response, ApiError>>,
                             api : Self::Client,
                             request: Self::Request) {
        match request {
            LibraryRequests::LoadPage(entity, offset, limit) => {
                log::info!("Async: Processing a Load Page Request");
                library_api::process_page_request(glib_sender, api, entity, offset, limit).await;
            }
        }
    }
}


impl LibraryApi {
    pub fn new(rt : ApiRuntime, api_address : String) -> Self {
        Self(ApiBase::new(rt, api_address))
    }

    pub fn load_tracks<CB>(&self, offset : i32, limit : i32, callback : CB) -> Result<(), ApiError>
    where CB : Fn(services::SimpleTrack) + 'static {
        let request = LibraryRequests::LoadPage(LibraryEntity::Track, offset, limit);
        self.0.request(request, move |response| {
            match response {
                LibraryResponse::Track(track) => callback(track),
                _ => unimplemented!("Received response other than track for a Track Request!")
            }
        })
    }
}

#[async_trait]
impl ApiClientBase for LibraryClient {
    async fn connect<D>(dst: D) -> Result<Self, Error>
        where D: TryInto<Endpoint>,
              D::Error: Into<StdError>,
              D: Send {
        LibraryClient::connect(dst).await
    }
}

///
/// Playback Controls API
///
#[derive(Clone, Debug)]
pub struct PlaybackApi(ApiBase<Self>);

#[async_trait]
impl ApiTypes for PlaybackApi {
    type Request = PlaybackRequest;
    type Response = PlaybackResponse;
    type Client = PlaybackClient;

    async fn process_request(glib_tx: Sender<Result<Self::Response, ApiError>>,
                             api: Self::Client,
                             request: Self::Request) {
        match request {
            PlaybackRequest::PlayTrack(track) => playback_api::play_track(glib_tx, api, track).await,
            PlaybackRequest::Play => playback_api::play(glib_tx, api).await,
            PlaybackRequest::Pause => playback_api::pause(glib_tx, api).await,
            PlaybackRequest::Next => playback_api::next(glib_tx, api).await,
            PlaybackRequest::Previous => playback_api::previous(glib_tx, api).await,
            PlaybackRequest::Seek(pos) => playback_api::seek(glib_tx, api, pos).await,

            PlaybackRequest::Shuffle => playback_api::shuffle(glib_tx, api).await,
            PlaybackRequest::Looping(mode) => playback_api::looping(glib_tx, api, mode).await,

            PlaybackRequest::CurrentState => playback_api::current_state(glib_tx, api).await,
            PlaybackRequest::StateUpdates => playback_api::state_updates(glib_tx, api).await,

            PlaybackRequest::QueueLoad(offset, limit) => playback_api::queue_load(glib_tx, api, offset, limit).await,
            PlaybackRequest::QueueAppend(track) => playback_api::queue_append(glib_tx, api, track).await,
            PlaybackRequest::QueuePrepend(track) => playback_api::queue_prepend(glib_tx, api, track).await,
            PlaybackRequest::QueueRemove(pos) => playback_api::queue_remove(glib_tx, api, pos).await,
            PlaybackRequest::QueueClear => playback_api::queue_clear(glib_tx, api).await,
        }
    }
}

impl PlaybackApi {
    pub fn new(rt : ApiRuntime, address : String) -> Self {
        Self(ApiBase::new(rt, address))
    }

    pub fn queue_load<CB>(&self, offset : i32, limit : i32, callback : CB) -> Result<(), ApiError>
    where CB : Fn(services::SimpleTrack) + 'static {
        let request = PlaybackRequest::QueueLoad(offset, limit);
        self.0.request(request, move |response| {
            match response {
                PlaybackResponse::QueueTrack(track) => callback(track),
                _ => unimplemented!("Received something other than Track for Queue Load!")
            }
        })
    }

    pub fn queue_append<CB>(&self, track_id : i32, callback : CB) -> Result<(), ApiError>
    where CB : Fn() + 'static {
        let request = PlaybackRequest::QueueAppend(track_id);
        self.0.request(request, move |response| {
            match response {
                PlaybackResponse::ActionConfirmed => callback(),
                _ => unimplemented!("Received something other than ActionConfirmed for Queue Add!")
            }
        })
    }

    pub fn play<CB>(&self, callback : CB) -> Result<(), ApiError>
    where CB : Fn(services::PlaybackStateResponse) + 'static {
        self.0.request(PlaybackRequest::Play, move |response| {
            match response {
                PlaybackResponse::CurrentState(state) => callback(state),
                _ => unimplemented!("Received something other than CurrentState for Play!")
            }
        })
    }

    pub fn play_track<CB>(&self, track_id : i32, callback : CB) -> Result<(), ApiError> 
    where CB : Fn(services::PlaybackStateResponse) + 'static {
        self.0.request(PlaybackRequest::PlayTrack(track_id), move |response| {
            match response {
                PlaybackResponse::CurrentState(state) => callback(state),
                _ => unimplemented!("Received something other than CurrentState for PlayTrack!")
            }
        })
    }

    pub fn pause<CB>(&self, callback : CB) -> Result<(), ApiError>
    where CB : Fn(services::PlaybackStateResponse) + 'static {
        self.0.request(PlaybackRequest::Pause, move |response| {
            match response {
                PlaybackResponse::CurrentState(state) => callback(state),
                _ => unimplemented!("Received something other than CurrentState for Pause!")
            }
        })
    }

    pub fn next<CB>(&self, callback : CB) -> Result<(), ApiError> 
    where CB : Fn(services::PlaybackStateResponse) + 'static {
        self.0.request(PlaybackRequest::Next, move |response| {
            match response {
                PlaybackResponse::CurrentState(state) => callback(state),
                _ => unimplemented!("Received something other than CurrentState for Next Track!")
            }
        })
    }

    pub fn current_state<CB>(&self, callback : CB) -> Result<(), ApiError>
    where CB : Fn(services::PlaybackStateResponse) + 'static {
        self.0.request(PlaybackRequest::CurrentState, move |response| {
            match response {
                PlaybackResponse::CurrentState(state) => callback(state),
                _ => unimplemented!("received something other than Current State for Current State!")
            }
        })
    }

    pub fn connect_state_update_notify<CB>(&self, callback : CB) -> Result<(), ApiError>
    where CB : Fn(services::PlaybackStateResponse) + 'static {
        self.0.request(PlaybackRequest::StateUpdates, move |response| {
            match response {
                PlaybackResponse::CurrentState(state) => callback(state),
                _ => unimplemented!("received something other than Current State for State Updates!")
            }
        })
    }
}

#[async_trait]
impl ApiClientBase for PlaybackClient {
    async fn connect<D>(dst: D) -> Result<Self, Error>
        where D: TryInto<Endpoint>,
              D::Error: Into<StdError>,
              D: Send {
        PlaybackClient::connect(dst).await
    }
}

#[derive(Debug, Clone)]
pub enum ApiError {
    Send(String),

    Request(String),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ApiError {
    fn from(e : tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Send(e.to_string())
    }
}