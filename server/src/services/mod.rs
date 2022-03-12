pub mod library;
pub mod tasks;
pub mod spotify_auth;
pub mod proposals;

pub mod definition {
    tonic::include_proto!("soundbase");
}