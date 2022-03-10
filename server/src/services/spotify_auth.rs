use tonic::{Request, Response, Status};
use crate::spotify::SpotifyApi;
use super::definition::spotify_auth_server::SpotifyAuth;
use super::definition::{SpotifyAuthBlank, SpotifyAuthString, SpotifyAuthUrls};

pub struct SpotifyAuthService {
    pub(crate) spotify : SpotifyApi
}

#[tonic::async_trait]
impl SpotifyAuth for SpotifyAuthService {
    async fn get_auth_url(&self, _request : Request<SpotifyAuthBlank>) -> Result<Response<SpotifyAuthUrls>, Status> {
        match self.spotify.get_auth_urls().await {
            Ok((auth_url, redir_url)) => {
                Ok(Response::new(SpotifyAuthUrls{
                    auth_url,
                    redir_url
                }))
            },
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }

    async fn send_auth_code(&self, request : Request<SpotifyAuthString>) -> Result<Response<SpotifyAuthBlank>, Status> {
        let code = &request.get_ref().value;
        println!("Received Auth Code '{:?}'", code);
        match self.spotify.finish_initialization_with_code(code).await {
            Ok(_) => Ok(Response::new(SpotifyAuthBlank{})),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }
}