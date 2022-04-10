use crate::api::ApiError;
use tonic::Request;

#[derive(Clone, Debug)]
pub enum PlaybackRequest {
    PlayTrack(i32),
    Play,
    Pause,
    Next,
    Previous,
    Seek(i64),

    Shuffle,
    Looping(LoopingMode),

    CurrentState,
    StateUpdates,

    QueueLoad(i32, i32),
    QueueAppend(i32),
    QueuePrepend(i32),
    QueueRemove(i32),
    QueueClear,
}

#[derive(Clone, Debug)]
pub enum PlaybackResponse {
    ActionConfirmed,
    CurrentState(super::services::PlaybackStateResponse),
    QueueTrack(super::services::SimpleTrack),
}

#[derive(Clone, Debug)]
pub enum LoopingMode {
    Off,
    One,
    All,
}

pub async fn play_track(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    track_id: i32,
) {
    let mut api = api.clone();
    let result = api
        .play_track(Request::new(super::services::PlaybackTrackRequest {
            track_id,
        }))
        .await;
    match result {
        Ok(response) => {
            let state_update = response.get_ref();
            log::info!("Transferring State {:?} to glib context!", state_update);
            glib_tx
                .send(Ok(PlaybackResponse::CurrentState(state_update.clone())))
                .expect("Failed to send to GLib Main Context!")
        }
        Err(e) => glib_tx
            .send(Err(ApiError::Request(e.to_string())))
            .expect("Failed to send to GLib Main Context!"),
    }
}

pub async fn play(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    let mut api = api.clone();
    let result = api
        .play(Request::new(super::services::PlaybackBlank {}))
        .await;
    match result {
        Ok(response) => {
            let state_update = response.get_ref();
            log::info!("Transferring State {:?} to glib context!", state_update);
            glib_tx
                .send(Ok(PlaybackResponse::CurrentState(state_update.clone())))
                .expect("Failed to send to GLib Main Context!")
        }
        Err(e) => glib_tx
            .send(Err(ApiError::Request(e.to_string())))
            .expect("Failed to send to GLib Main Context!"),
    }
}

pub async fn pause(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    let mut api = api.clone();
    let result = api
        .pause(Request::new(super::services::PlaybackBlank {}))
        .await;
    match result {
        Ok(response) => {
            let state_update = response.get_ref();
            glib_tx
                .send(Ok(PlaybackResponse::CurrentState(state_update.clone())))
                .expect("Failed to send to GLib Main Context!")
        }
        Err(e) => glib_tx
            .send(Err(ApiError::Request(e.to_string())))
            .expect("Failed to send to GLib Main Context!"),
    }
}

pub async fn next(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    let mut api = api.clone();
    let result = api
        .next(Request::new(super::services::PlaybackBlank {}))
        .await;
    match result {
        Ok(response) => {
            let state_update = response.get_ref();
            glib_tx
                .send(Ok(PlaybackResponse::CurrentState(state_update.clone())))
                .expect("Failed to send to GLib Main Context!")
        }
        Err(e) => glib_tx
            .send(Err(ApiError::Request(e.to_string())))
            .expect("Failed to send to GLib Main Context!"),
    }
}

pub async fn previous(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    todo!()
}

pub async fn seek(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    target_pos: i64,
) {
    todo!()
}

pub async fn shuffle(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    todo!()
}

pub async fn looping(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    target_mode: LoopingMode,
) {
    todo!()
}

pub async fn current_state(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    let mut api = api.clone();
    let result = api
        .current_state(Request::new(super::services::PlaybackBlank {}))
        .await;
    match result {
        Ok(response) => {
            let state_update = response.get_ref();
            log::info!("Transferring State {:?} to glib context!", state_update);
            glib_tx
                .send(Ok(PlaybackResponse::CurrentState(state_update.clone())))
                .expect("Failed to send to GLib Main Context!")
        }
        Err(e) => glib_tx
            .send(Err(ApiError::Request(e.to_string())))
            .expect("Failed to send to GLib Main Context!"),
    }
}

pub async fn state_updates(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    let mut api = api.clone();
    let result = api
        .state_updates(Request::new(super::services::PlaybackBlank {}))
        .await;
    match result {
        Ok(r) => {
            let mut result_stream = r.into_inner();
            loop {
                let msg = result_stream.message().await;
                match msg {
                    Ok(m) => match m {
                        Some(response) => {
                            log::info!("Transferring State {:?} to glib context!", response);
                            glib_tx
                                .send(Ok(PlaybackResponse::CurrentState(response.clone())))
                                .expect("Failed to send to GLib Main Context!")
                        }
                        None => {
                            log::info!("End of State Update Stream reached!");
                            break;
                        }
                    },
                    Err(e) => glib_tx
                        .send(Err(ApiError::Request(format!(
                            "Stream terminated with '{:?}'",
                            e
                        ))))
                        .expect("Failed to send to GLib Main Context!"),
                }
            }
        }
        Err(e) => panic!("Error fetching from backend {:?}", e),
    }
}

pub async fn queue_load(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    offset: i32,
    limit: i32,
) {
    let mut api = api.clone();
    let result = api
        .get_queue(Request::new(super::services::GetQueueRequest {
            offset,
            limit,
        }))
        .await;
    match result {
        Ok(r) => {
            let mut result_stream = r.into_inner();
            loop {
                let msg = result_stream.message().await;
                match msg {
                    Ok(m) => match m {
                        Some(track) => glib_tx
                            .send(Ok(PlaybackResponse::QueueTrack(track)))
                            .expect("Failed to send to GLib!"),
                        None => {
                            log::info!("Reached end of Queue load stream!");
                            break;
                        }
                    },
                    Err(e) => {
                        glib_tx
                            .send(Err(ApiError::Request(format!(
                                "Stream terminated with '{:?}'",
                                e
                            ))))
                            .expect("Failed to send to GLib!");
                        break;
                    }
                }
            }
        }
        Err(e) => panic!("Error fetching from Backend! ({:?})", e),
    }
}

pub async fn queue_append(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    track_id: i32,
) {
    let mut api = api.clone();
    let result = api
        .append_to_queue(Request::new(super::services::ToQueueRequest { track_id }))
        .await;
    match result {
        Ok(_r) => {
            glib_tx
                .send(Ok(PlaybackResponse::ActionConfirmed))
                .expect("Failed to send to GLib!");
        }
        Err(e) => glib_tx
            .send(Err(ApiError::Request(format!(
                "Failed to add Track to Queue! ('{:?}'",
                e
            ))))
            .expect("Failed to send to GLib!"),
    }
}

pub async fn queue_prepend(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    track_id: i32,
) {
    todo!()
}

pub async fn queue_remove(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
    queue_pos: i32,
) {
    todo!()
}

pub async fn queue_clear(
    glib_tx: gtk4::glib::Sender<Result<PlaybackResponse, ApiError>>,
    api: super::PlaybackClient,
) {
    todo!()
}
