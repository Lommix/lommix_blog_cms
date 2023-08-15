#![allow(unused)]

use axum::{
    body::{Body, StreamBody},
    extract::{connect_info::IntoMakeServiceWithConnectInfo, Host, Query, State},
    handler::HandlerWithoutStateExt,
    http::{Request, Response, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    BoxError, Json, Router, ServiceExt,
};

use axum_server::tls_rustls::RustlsConfig;
use chrono::NaiveDateTime;
use clap::{Args, Parser};
use minijinja::{context, value::Value};
use serde::{Deserialize, Serialize};
use std::{
    default,
    ffi::OsStr,
    io::BufReader,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::store::{articles::Article, paragraphs::Paragraph};
use dotenv::dotenv;
use store::SchemaUp;

mod api;
mod auth;
mod pages;
mod store;
mod util;

const TEMPLATE_DIR: &str = "templates";
const PAGE_DIR: &str = "pages";
const TEMPLATE_EXTENSION: &str = "html";
const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

// --------------------------------------------------------
// shared state
// --------------------------------------------------------
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum UserState {
    #[default]
    Unknown,
    User,
    Admin,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Session {
    pub id: u128,
    pub user_state: UserState,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalContext {
    pub user_state: UserState,
}

#[derive(Debug)]
pub struct SharedState {
    pub db: rusqlite::Connection,
    pub templates: minijinja::Environment<'static>,
    pub sessions: RwLock<Vec<Session>>,
}

unsafe impl Send for SharedState {}
unsafe impl Sync for SharedState {}

#[derive(Parser)]
enum Command {
    Init,
    Dev,
    Prod,
}

// --------------------------------------------------------
// main
// --------------------------------------------------------
#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_path = std::env::var("DATABASE_PATH").expect("DATABASE_PATH must be set");
    let db = rusqlite::Connection::open(&db_path).expect("Failed to open database");

    let state = Arc::new(SharedState {
        db,
        templates: load_templates(),
        sessions: RwLock::new(Vec::new()),
    });

    let cmd = Command::parse();

    match cmd {
        Command::Init => {
            Article::up(&state.db).unwrap();
            Paragraph::up(&state.db).unwrap();
        }
        Command::Dev => {
            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            tracing::info!("listening on {}", addr);
            let app = setup_router(state);
            axum::Server::bind(&addr).serve(app).await.unwrap();
        }
        Command::Prod => {
            let cert_path = std::env::var("SSL_CERT").expect("CERT_PATH must be set");
            let key_path = std::env::var("SSL_KEY").expect("KEY_PATH must be set");

            let config =
                RustlsConfig::from_pem_file(PathBuf::from(cert_path), PathBuf::from(key_path))
                    .await
                    .expect("failed to load cert");

            let mut addr = SocketAddr::from(([0, 0, 0, 0], HTTPS_PORT));

            tracing::info!("listening on {}", addr);

            let app = setup_router(state);

            tokio::spawn(redirect_http_to_https());
            axum_server::bind_rustls(addr, config)
                .serve(app)
                .await
                .unwrap();
        }
    }
}

fn setup_router(state: Arc<SharedState>) -> IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
    Router::new()
        .route("/static/*asset", get(asset_handle))
        .nest("/", pages::page_routes())
        .nest("/api", api::api_routes())
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>()
}

async fn redirect_http_to_https() {
    fn make_https(host: String, uri: Uri) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&HTTP_PORT.to_string(), &HTTPS_PORT.to_string());
        parts.authority = Some(https_host.parse()?);
        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], HTTP_PORT));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    let router = Router::new()
        .route_service("/", get(redirect))
        .route_service("/*any", get(redirect))
        .into_make_service();

    axum::Server::from_tcp(listener.into_std().unwrap())
        .unwrap()
        .serve(router)
        .await
        .unwrap();
}

// --------------------------------------------------------
// static file handler
// lommix.de/static
// --------------------------------------------------------
async fn asset_handle(uri: Uri) -> impl IntoResponse {
    let path = PathBuf::from(uri.path())
        .iter()
        .skip(1)
        .collect::<PathBuf>();

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(e) => {
            println!("error opening file: {}", e);
            return Err((StatusCode::NOT_FOUND, "file not found"));
        }
    };

    let content_type = match mime_guess::from_path(&path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err((StatusCode::BAD_REQUEST, "unknown content type"));
        }
    };

    tracing::info!("serving asset: {:?}", path.to_str());

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let headers = [(axum::http::header::CONTENT_TYPE, content_type)];

    Ok((headers, body))
}

fn load_dir(dir: PathBuf) -> Vec<(String, PathBuf)> {
    let mut files = Vec::new();
    std::fs::read_dir(dir)
        .expect("unable to load dir")
        .for_each(|file| {
            let path = file.unwrap().path();
            if path.is_dir() {
                files.append(&mut load_dir(path));
            } else if path.is_file() && path.extension() == Some(OsStr::new(TEMPLATE_EXTENSION)) {
                let route = path
                    .components()
                    .skip_while(|c| c.as_os_str() != TEMPLATE_DIR)
                    .collect::<PathBuf>()
                    .to_string_lossy()
                    .to_string();
                files.push((format!("/{}", route), path));
            }
        });
    files
}

fn load_templates() -> minijinja::Environment<'static> {
    let templates = load_dir(TEMPLATE_DIR.into());

    let mut env = minijinja::Environment::new();

    templates.iter().for_each(|(_, path)| {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        let template = std::fs::read_to_string(path).unwrap();

        let name = path
            .components()
            .skip(1)
            .collect::<PathBuf>()
            .to_string_lossy()
            .to_string();

        println!("loaded template: {} line: {}", file_name, name);

        env.add_filter("date", date_format);
        env.add_template_owned(name, template)
            .expect("error loading template");
    });

    env
}

fn date_format(state: &minijinja::State, value: i64) -> String {
    let time = match NaiveDateTime::from_timestamp_opt(value, 0) {
        Some(time) => time.format("%d. %B %Y").to_string(),
        None => return "".to_string(),
    };

    time
}
