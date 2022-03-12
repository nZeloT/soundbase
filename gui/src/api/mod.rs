use tokio::runtime::Runtime;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use mpsc::{Sender as TokioSender};
use tonic::Request;
use crate::tracks_page::TracksPageMsg;
use crate::TracksPageModel;

use services::{ListEntitiesRequest, LibraryEntities, SimpleLibraryEntityResponse, simple_library_entity_response, SimpleTrack};

pub mod services {
    tonic::include_proto!("soundbase");
}

type API = services::library_client::LibraryClient<tonic::transport::Channel>;
pub struct AsyncLibraryHandler {
    _rt : Runtime,
    sender : TokioSender<AsyncLibraryHandlerMsg>
}

#[derive(Debug)]
pub enum AsyncLibraryHandlerMsg {
    LoadPage(AsyncLibraryKind, i32, i32)
}

#[derive(Debug)]
pub enum AsyncLibraryKind {
    Track
}

impl relm4::MessageHandler<TracksPageModel> for AsyncLibraryHandler {
    type Msg = AsyncLibraryHandlerMsg;
    type Sender = TokioSender<AsyncLibraryHandlerMsg>;

    fn init(_parent_model : &TracksPageModel, parent_sender : relm4::Sender<TracksPageMsg>) -> Self {
        let api_target = match std::env::var("API_URL") {
            Ok(url) => url,
            Err(_) => "http://philly.local:3333".to_string()
        };
        println!("Found API target: '{:?}'", api_target);
        let (sender, mut rx) = mpsc::channel::<AsyncLibraryHandlerMsg>(10);
        let rt = Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.spawn(async move {
            let api = services::library_client::LibraryClient::connect(api_target).await
                .expect("Couldn't connect to API backend!");
            println!("Connected to backend!");
            while let Some(msg) = rx.recv().await {
                let parent_sender = parent_sender.clone();
                let mut _api = api.clone();
                tokio::spawn(async move {
                    //TODO handle message and send message using parent_sender
                    println!("Handling async Message");
                    match msg {
                        AsyncLibraryHandlerMsg::LoadPage(kind, offset, limit) => {
                            AsyncLibraryHandler::handle_load_page(
                                &parent_sender, &mut _api, kind, offset, limit).await;
                        }
                    }
                });
            }
            println!("Disconnected from backend");
        });

        AsyncLibraryHandler{
            _rt : rt,
            sender
        }
    }

    fn send(&self, msg : Self::Msg) {
        self.sender.blocking_send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}

impl AsyncLibraryHandler {

    async fn handle_load_page(receiver : &relm4::Sender<TracksPageMsg>, api : &mut API,
                              kind : AsyncLibraryKind, offset : i32, limit : i32) {
        match kind {
            AsyncLibraryKind::Track => {
                AsyncLibraryHandler::handle_load_page_tracks(receiver, api, offset, limit).await;
            }
            //TODO: add further entities
        }
    }

    async fn handle_load_page_tracks(receiver : &relm4::Sender<TracksPageMsg>, api : &mut API,
                                     offset : i32, limit : i32) {

        let result = api.list(Request::new(ListEntitiesRequest{
            entity: LibraryEntities::Track as i32,
            offset,
            limit
        })).await;

        match result {
            Ok(r) => {
                let mut stream = r.into_inner();
                loop {
                    let msg = stream.message().await;
                    match msg {
                        Ok(m) => {
                            match m {
                                Some(response) => {
                                    let simple_track = AsyncLibraryHandler::unpack_simple_track(response);
                                    println!("Sending Track to Tracks Page: '{:?}'", simple_track);
                                    relm4::send!(receiver, TracksPageMsg::AddApiTrack(simple_track))
                                },
                                None => {
                                    // Stream was terminated gracefully as everything wa received
                                    print!("Reached Stream End");
                                    break;
                                }
                            }
                        },
                        Err(e) => {
                            println!("Stream terminated with '{:?}'", e);
                            break; //terminate loop
                        }
                    }
                }
            },
            Err(e) => panic!("Error fetching from backend! {:?}", e)
        }
    }

    fn unpack_simple_track(response : SimpleLibraryEntityResponse) -> SimpleTrack {
        let entity_response = response
            .library_entities
            .unwrap();
        match entity_response {
            simple_library_entity_response::LibraryEntities::Track(simple_track) => simple_track,
            _ => unimplemented!()
        }
    }
}