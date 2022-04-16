use crate::api::ApiError;
use tonic::Request;

#[derive(Clone, Debug)]
pub enum LibraryEntity {
    Track,
    Album,
}

#[derive(Clone, Debug)]
pub enum LibraryRequests {
    LoadPage(LibraryEntity, i32, i32),
}

#[derive(Clone, Debug)]
pub enum LibraryResponse {
    Track(super::services::SimpleTrack),
    Album(super::services::SimpleAlbum),
    Artist(super::services::SimpleArtist),
}

pub async fn process_page_request(
    glib_tx: gtk4::glib::Sender<Result<LibraryResponse, ApiError>>,
    api: super::LibraryClient,
    entity: LibraryEntity,
    offset: i32,
    limit: i32,
) {
    match entity {
        LibraryEntity::Track => process_page_track_request(glib_tx, api, offset, limit).await,
        LibraryEntity::Album => process_page_album_request(glib_tx, api, offset, limit).await,
    }
}

async fn process_page_track_request(
    glib_tx: gtk4::glib::Sender<Result<LibraryResponse, ApiError>>,
    api: super::LibraryClient,
    offset: i32,
    limit: i32,
) {
    log::info!(
        "Async: Processing a Track Page Request with offset {} and limit {}",
        offset,
        limit
    );
    handle_page_request(
        api,
        LibraryEntity::Track,
        offset,
        limit,
        move |response| match response {
            Ok(api_response) => {
                log::info!("Async: Received a response for request. Going to unpack.");
                let simple_track = unpack_track(api_response);
                let lib_response = LibraryResponse::Track(simple_track);
                glib_tx
                    .send(Ok(lib_response))
                    .expect("Failed to send to GLib!")
            }
            Err(e) => glib_tx.send(Err(e)).expect("Failed to send to Glib!"),
        },
    )
    .await
}

async fn process_page_album_request(
    glib_tx: gtk4::glib::Sender<Result<LibraryResponse, ApiError>>,
    api: super::LibraryClient,
    offset: i32,
    limit: i32,
) {
    log::info!(
        "Async: Processing a Album Page Request with offset {} and limit {}",
        offset,
        limit
    );
    handle_page_request(
        api,
        LibraryEntity::Album,
        offset,
        limit,
        move |response| match response {
            Ok(api_response) => {
                log::info!("Async: Received a response for request. Going to unpack.");
                let simple_album = unpack_album(api_response);
                let lib_response = LibraryResponse::Album(simple_album);
                glib_tx
                    .send(Ok(lib_response))
                    .expect("Failed to send to GLib Main Context!")
            }
            Err(e) => glib_tx
                .send(Err(e))
                .expect("Failed to send to GLib Maint Context!"),
        },
    )
    .await
}

async fn handle_page_request<CB>(
    api: super::LibraryClient,
    entity: LibraryEntity,
    offset: i32,
    limit: i32,
    process_response: CB,
) where
    CB: Fn(Result<super::services::SimpleLibraryEntityResponse, ApiError>),
{
    let mut api = api.clone();
    let result = api
        .list(Request::new(super::services::ListEntitiesRequest {
            entity: super::services::LibraryEntities::from(entity) as i32,
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
                        Some(response) => process_response(Ok(response)),
                        None => {
                            println!("Reached Stream End");
                            break;
                        }
                    },
                    Err(e) => {
                        process_response(Err(ApiError::Request(format!(
                            "Stream terminated with '{:?}'",
                            e
                        ))));
                        break;
                    }
                }
            }
        }
        Err(e) => panic!("Error fetching from Backend! ({:?})", e),
    }
}

fn unpack_track(
    response: super::services::SimpleLibraryEntityResponse,
) -> super::services::SimpleTrack {
    let entity_response = response.library_entities.unwrap();
    match entity_response {
        super::services::simple_library_entity_response::LibraryEntities::Track(simple_track) => {
            simple_track
        },
        _ => panic!("Expected Track!"),
    }
}

fn unpack_album(
    response: super::services::SimpleLibraryEntityResponse,
) -> super::services::SimpleAlbum {
    let entity_response = response.library_entities.unwrap();
    match entity_response {
        super::services::simple_library_entity_response::LibraryEntities::Album(simple_album) => {
            simple_album
        },
        _ => panic!("Expected Album!"),
    }
}

impl From<LibraryEntity> for super::services::LibraryEntities {
    fn from(entity: LibraryEntity) -> Self {
        match entity {
            LibraryEntity::Track => Self::Track,
            LibraryEntity::Album => Self::Album
        }
    }
}
