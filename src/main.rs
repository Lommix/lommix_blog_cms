#![allow(unused)]

use axum::{
    body::{Body, StreamBody},
    extract::{Query, State},
    http::{Request, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
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

// --------------------------------------------------------
// shared state
// --------------------------------------------------------
#[derive(Debug,Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
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
    Run,
}

// --------------------------------------------------------
// main
// --------------------------------------------------------
#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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
        Command::Run => {
            println!("listening on {}", addr);
            let app = Router::new()
                .route("/static/*asset", get(asset_handle))
                .nest("/", pages::page_routes())
                .nest("/api", api::api_routes())
                .with_state(state)
                .into_make_service();
            axum::Server::bind(&addr).serve(app).await.unwrap();
        }
    }
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
            } else if path.is_file() && path.extension() == Some(&OsStr::new(TEMPLATE_EXTENSION)) {
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

        let template = std::fs::read_to_string(&path).unwrap();

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
    let time = chrono::NaiveDateTime::from_timestamp(value, 0);
    let out = time.format("%d. %B %Y").to_string();
    format!("{}", &out)
}
