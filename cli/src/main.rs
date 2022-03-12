use std::collections::HashMap;
use std::net::SocketAddr;

use clap::{Parser, Subcommand, Args};
use tonic::Request;
use url::Url;
use warp::Filter;

pub mod services {
    tonic::include_proto!("soundbase");
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {

    server : String,

    #[clap(subcommand)]
    command : Commands
}

#[derive(Subcommand)]
enum Commands {
    AuthSpotify,
    Tasks(TasksParam)
}

#[derive(Args)]
struct TasksParam {
    #[clap(subcommand)]
    subcommand : TaskCommands
}

#[derive(Subcommand)]
enum TaskCommands {
    AlbumOfWeek,
    ChartsOfWeek,
    SyncFromSpotify
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let server_url = cli.server.clone();

    match &cli.command {
        &Commands::AuthSpotify => do_spotify_auth(server_url).await,
        Commands::Tasks(params) => {
            match &params.subcommand {
                TaskCommands::ChartsOfWeek => tasks::do_fetch_charts_of_week(server_url).await,
                TaskCommands::AlbumOfWeek => tasks::do_fetch_album_of_week(server_url).await,
                TaskCommands::SyncFromSpotify => tasks::do_sync_from_spotify(server_url).await
            }
        }
    }
}

async fn do_spotify_auth(server : String) -> Result<(), Box<dyn std::error::Error>> {
    let mut auth_client = services::spotify_auth_client::SpotifyAuthClient::connect(server).await?;

    //1. get the auth url
    let response = auth_client.get_auth_url(Request::new(services::SpotifyAuthBlank {})).await?;
    let urls = response.get_ref();
    let auth_url = urls.auth_url.clone();
    let redir_url = urls.redir_url.clone();
    println!("Authenticate with spotify by visiting '{}'", auth_url);

    println!("Redir URL '{}'", redir_url);
    let redir_url = Url::parse(&redir_url)?;
    println!("Parsed redir Url '{}'", redir_url);
    let path_segments = redir_url.path_segments()
        .map(|s| s.collect::<Vec<_>>()).unwrap();
    let host = redir_url.host_str().unwrap();
    let port = redir_url.port().unwrap();

    let mut path = warp::any().boxed();
    for segment in path_segments {
        path = path.and(warp::path(segment.to_string())).boxed();
    }
    path = path.and(warp::path::end()).boxed();

    println!("\tCompound path: '{:?}'", path);

    //2. launch a webserver listening for callback
    let (term_tx, mut term_rx) = tokio::sync::mpsc::channel(1);
    let (code_tx, mut code_rx) = tokio::sync::mpsc::channel(1);
    let route = warp::any()
        .and(path)
        .and(warp::get())
        .and(warp::query::<HashMap<String,String>> ())
        .and(warp::any().map(move || code_tx.clone()))
        .and(warp::any().map(move || term_tx.clone()))
        .and_then(handle_auth_callback);

    let socket_addr : SocketAddr = (host.to_string() + ":" + &port.to_string()).parse()?;
    println!("\tObtained Socket Address: '{:?}'", socket_addr);

    let (_addr, server) = warp::serve(route).bind_with_graceful_shutdown(socket_addr, async move{
        term_rx.recv().await;
    });

    tokio::task::spawn(server).await?;

    //3. send the received code to the server
    let code_opt = code_rx.recv().await.unwrap();
    match code_opt {
        Some(code) => {
            let _ = auth_client.send_auth_code(Request::new(services::SpotifyAuthString {
                value: code
            })).await?;
        },
        None => println!("Authentication with spotify failed; Try again!")
    }

    Ok(())
}

async fn handle_auth_callback(query: HashMap<String, String>, code_tx : tokio::sync::mpsc::Sender<Option<String>>,
                              term_tx : tokio::sync::mpsc::Sender<()>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code_opt = query.get("code").map(|code| code.to_string());
    println!("Received Auth code '{:?}'", code_opt);
    let _ = code_tx.send(code_opt).await;
    let _ = term_tx.send(()).await;

    Ok(warp::reply::reply())
}

mod tasks {
    use tonic::Request;

    pub async fn do_sync_from_spotify(server : String) -> Result<(), Box<dyn std::error::Error>> {
        let mut tasks_client = super::services::tasks_client::TasksClient::connect(server).await?;
        let _ = tasks_client.update_from_spotify(
            Request::new(super::services::TasksBlank{})).await?;

        Ok(())
    }

    pub async fn do_fetch_album_of_week(server : String) -> Result<(), Box<dyn std::error::Error>> {
        let mut tasks_client = super::services::tasks_client::TasksClient::connect(server).await?;
        let _ = tasks_client.fetch_album_of_week(
            Request::new(super::services::TasksBlank{})).await?;
        Ok(())
    }

    pub async fn do_fetch_charts_of_week(server : String) -> Result<(), Box<dyn std::error::Error>> {
        let mut tasks_client = super::services::tasks_client::TasksClient::connect(server).await?;
        let _ = tasks_client.fetch_charts(
            Request::new(super::services::TasksBlank{})).await?;
        Ok(())
    }
}