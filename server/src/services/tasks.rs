use tonic::{Request, Response, Status};
use crate::db_new::DbApi;
use crate::spotify::SpotifyApi;

use super::definition::tasks_server::Tasks;
use super::definition::TasksBlank;

pub struct TasksService {
    pub(crate) db : DbApi,
    pub(crate) spotify : SpotifyApi
}

#[tonic::async_trait]
impl Tasks for TasksService {
    async fn fetch_charts(&self, _request : Request<TasksBlank>) -> Result<Response<TasksBlank>, Status> {
        crate::tasks::launch_fetch_charts(&self.db);
        Ok(Response::new(TasksBlank{}))
    }

    async fn fetch_album_of_week(&self, _request : Request<TasksBlank>) -> Result<Response<TasksBlank>, Status> {
        crate::tasks::launch_fetch_albums_of_week(&self.db);
        Ok(Response::new(TasksBlank{}))
    }

    async fn update_from_spotify(&self, _request : Request<TasksBlank>) -> Result<Response<TasksBlank>, Status> {
        crate::tasks::launch_spotify_import(&self.db, &self.spotify);
        Ok(Response::new(TasksBlank{}))
    }
}