use serde::{Serialize, Deserialize};
use serde_json;
use crate::handlers::Spotify;
use crate::error::{Result, SoundbaseError};
use std::time::Instant;

#[derive(Serialize,Deserialize,Debug,Clone)]
struct AccessTokenSuccessResponse {
    origin: String,
    access_token: String,
    expires_in: u64,
    #[serde(default)]
    refresh_token: Option<String>
}

#[derive(Serialize,Debug,Clone)]
struct AccessTokenErrorResponse {
    origin: String,
    error: String
}

pub async fn get_token_from_code(wrapper: Spotify, query: String) -> String {
    let mut spotify = wrapper.write().await;
    match spotify.finish_initialization_with_code(query.as_str()).await {
        Ok(_) => {
            let success = success_response(&spotify).await;
            serde_json::to_string(&success).unwrap()
        },
        Err(e) => {
            let error = AccessTokenErrorResponse {
                origin: "auth_spotify".to_owned(),
                error: e.msg
            };

            serde_json::to_string(&error).unwrap()
        }
    }
}

pub async fn get_token_from_refresh_token(wrapper: Spotify) -> Result<String> {
    let mut spotify = wrapper.write().await;
    spotify.force_token_update().await?;

    let success = success_response(&spotify).await;
    Ok(serde_json::to_string(&success)?)
}

async fn success_response(spotify: &crate::model::spotify::Spotify) -> AccessTokenSuccessResponse {
    let (access_token, expiry) = spotify.get_current_access_token().await;
    let refresh_token = spotify.get_refresh_token().await;

    let expires_in = (expiry - Instant::now()).as_secs();

    //in case we get here, we already know success
    AccessTokenSuccessResponse {
        origin: "auth_spotify".to_owned(),
        access_token,
        expires_in,
        refresh_token
    }
}