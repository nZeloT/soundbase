use async_trait::async_trait;
use futures::Future;

use crate::playback::PlaybackError;
use super::Player;

#[derive(Clone)]
pub struct LocalPlayer {

}

impl LocalPlayer{
    pub fn new() -> Self {
        Self{}
    }
}

#[async_trait]
impl Player for LocalPlayer {
    async fn connect_player_events(&mut self, tx: tokio::sync::mpsc::UnboundedSender<super::PlayerEvent>) {
        todo!()
    }

    async fn start(&self, track_ident: &str) -> Result<(), PlaybackError> {
        todo!()
    }

    async fn resume(&self) -> Result<(), PlaybackError> {
        todo!()
    }

    async fn pause(&self) -> Result<(), PlaybackError> {
        todo!()
    }

    async fn seek(&self, target_pos_ms: i64) -> Result<(), PlaybackError> {
        todo!()
    }
}