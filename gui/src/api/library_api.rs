use tonic::Request;
use crate::api::ApiError;

#[derive(Clone, Debug)]
pub enum LibraryEntity {
    Track
}

#[derive(Clone, Debug)]
pub enum LibraryRequests {
    LoadPage(LibraryEntity, i32, i32)
}

#[derive(Clone, Debug)]
pub enum LibraryResponse {
    Track(super::services::SimpleTrack),
    Album(super::services::SimpleAlbum),
    Artist(super::services::SimpleArtist)
}

pub async fn process_page_request(glib_tx: gtk4::glib::Sender<Result<LibraryResponse, ApiError>>, api: super::LibraryClient,
                                  entity: LibraryEntity, offset: i32, limit: i32) {
    match entity {
        LibraryEntity::Track => process_page_track_request(glib_tx, api, offset, limit).await
    }
}

async fn process_page_track_request(glib_tx: gtk4::glib::Sender<Result<LibraryResponse, ApiError>>,
                                    api: super::LibraryClient, offset: i32, limit: i32) {

    log::info!("Async: Processing a Track Page Request with offset {} and limit {}", offset, limit);
    handle_page_request(api, LibraryEntity::Track, offset, limit, move |response| {
        match response {
            Ok(api_response) => {
                log::info!("Async: Received a response for request. Going to unpack.");
                let simple_track = unpack_track(api_response);
                let lib_response = LibraryResponse::Track(simple_track);
                glib_tx.send(Ok(lib_response)).expect("Failed to send to GLib!")
            },
            Err(e) => glib_tx.send(Err(e)).expect("Failed to send to Glib!")
        }
    }).await
}

async fn handle_page_request<CB>(api: super::LibraryClient, entity: LibraryEntity,
                                 offset: i32, limit: i32, process_response : CB)
where CB : Fn(Result<super::services::SimpleLibraryEntityResponse, ApiError>)  {
    let mut api = api.clone();
    let result = api.list(Request::new(
        super::services::ListEntitiesRequest {
            entity: super::services::LibraryEntities::from(entity) as i32,
            offset,
            limit,
        }
    )).await;

    match result {
        Ok(r) => {
            let mut result_stream = r.into_inner();
            loop {
                let msg = result_stream.message().await;
                match msg {
                    Ok(m) => {
                        match m {
                            Some(response) => process_response(Ok(response)),
                            None => {
                                println!("Reached Stream End");
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        process_response(Err(ApiError::Request(format!("Stream terminated with '{:?}'", e))));
                        break;
                    }
                }
            }
        },
        Err(e) => panic!("Error fetching from Backend! ({:?})", e)
    }
}

fn unpack_track(response : super::services::SimpleLibraryEntityResponse) -> super::services::SimpleTrack {
    let entity_response = response.library_entities.unwrap();
    match entity_response {
        super::services::simple_library_entity_response::LibraryEntities::Track(simple_track) => simple_track,
        _ => panic!("Expected Track!")
    }
}

impl From<LibraryEntity> for super::services::LibraryEntities {
    fn from(entity : LibraryEntity) -> Self {
        match entity {
            LibraryEntity::Track => Self::Track
        }
    }
}
