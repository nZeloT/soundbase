use std::fmt::Debug;
use std::sync::Arc;
use adw::glib;
use crate::api::{ApiError};
use async_trait::async_trait;
use tonic::codegen::StdError;

#[async_trait]
pub trait ApiTypes {
    type Request : Debug + Clone + Send + 'static;
    type Response : Debug + Clone + Send + 'static;
    type Client : ApiClientBase;

    async fn process_request(glib_tx : gtk4::glib::Sender<Result<Self::Response, ApiError>>,
                             api : Self::Client,
                             request : Self::Request);
}

#[async_trait]
pub trait ApiClientBase : Clone + Send + 'static {
    async fn connect<D>(dst : D) -> Result<Self, tonic::transport::Error>
        where D: std::convert::TryInto<tonic::transport::Endpoint>,
              D::Error: Into<StdError>,
              D: Send;
}

#[derive(Clone, Debug)]
pub struct ApiRuntime(Arc<tokio::runtime::Runtime>);
impl ApiRuntime {
    pub fn new() -> Self {
        Self(Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap()))
    }
}

#[derive(Clone, Debug)]
struct ApiRequest<T, R> {
    result_sender : gtk4::glib::Sender<Result<R, ApiError>>,
    request : T
}

impl<T, R> ApiRequest<T, R> {
    fn new(request : T, receiver : gtk4::glib::Sender<Result<R, ApiError>>) -> Self {
        Self {
            request,
            result_sender : receiver
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiBase<API>
    where API : ApiTypes + Clone {
    _rt : ApiRuntime,

    ///
    /// Used to blocking send messages from the Api interface used by the UI
    /// to the asynchronous ApiProcessor holding the api connection and
    /// executing the request
    ///
    sender : tokio::sync::mpsc::UnboundedSender<ApiRequest<API::Request, API::Response>>
}

impl<API> ApiBase<API>
where API : ApiTypes + Clone {

    pub fn new(rt : ApiRuntime, address : String) -> Self {
        let (tx, mut rx) : (tokio::sync::mpsc::UnboundedSender<ApiRequest<API::Request, API::Response>>,
                            tokio::sync::mpsc::UnboundedReceiver<ApiRequest<API::Request, API::Response>>)
            = tokio::sync::mpsc::unbounded_channel();

        rt.0.spawn(async move {
            let api = API::Client::connect(address).await
                .expect("Couldn't connect to API!");
            log::info!("Established new Backend Connection");

            while let Some(api_request) = rx.recv().await {
                log::info!("Async: Received a new API request.");
                let mut _api = api.clone();
                tokio::spawn(async move {
                    log::info!("Async: Processing an API Request");

                    let request = api_request.request;
                    let glib_tx : gtk4::glib::Sender<Result<API::Response, ApiError>> = api_request.result_sender;

                    API::process_request(glib_tx, _api, request).await;
                });
            }
        });

        Self{
            _rt : rt,
            sender: tx
        }
    }

    pub fn request<CB>(&self, request : API::Request, callback : CB) -> Result<(), ApiError>
    where CB : Fn(API::Response) + 'static {
        let (glib_tx, glib_rx) = gtk4::glib::MainContext::channel(gtk4::glib::PRIORITY_DEFAULT);
        let request = ApiRequest::new(
            request,
            glib_tx
        );

        {
            let context = gtk4::glib::MainContext::default();
            let _guard = context.acquire().expect("Couldn't acquire glib main context!");
            glib_rx.attach(Some(&context), move |response : Result<API::Response, ApiError>| {
                log::info!("Handling API response on GLib Main Context");
                match response {
                    Ok(response) => callback(response),
                    Err(e) => log::error!("Found API Error: {:?}", e)
                }

                glib::Continue(true)
            });
        }

        log::info!("Sending Request '{:?}' to async API handler.", request);
        self.sender.send(request)?;
        Ok(())
    }
}
