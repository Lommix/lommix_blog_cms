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
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    io::BufReader,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use crate::store::{articles::Article, paragraphs::Paragraph};
use dotenv::dotenv;
use store::SchemaUp;

mod api;
mod store;

const TEMPLATE_DIR: &str = "templates";
const PAGE_DIR: &str = "pages";
const TEMPLATE_EXTENSION: &str = "html";

// --------------------------------------------------------
// shared state
// --------------------------------------------------------
pub enum UserState {
    Unknown,
    User,
    Admin,
}

pub struct Session {
    pub id: u128,
    pub user_state: UserState,
}

pub struct Shared {
    pub db: rusqlite::Connection,
    pub templates: minijinja::Environment<'static>,
    pub sessions: RwLock<Vec<Session>>,
}

unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

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

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let db_path = std::env::var("DATABASE_PATH").expect("DATABASE_PATH must be set");
    let db = rusqlite::Connection::open(&db_path).expect("Failed to open database");

    let state = Arc::new(Shared {
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
                .route("/favicon.ico", get(favicon))
                .route("/static/*asset", get(asset_handle))
                .route("/*url", get(default_handle))
                .route("/", get(default_handle))
                .nest("/api", api::api_routes())
                .with_state(state)
                .into_make_service();
            axum::Server::bind(&addr).serve(app).await.unwrap();
        }
    }
}

async fn favicon() -> impl IntoResponse {
    Html("".to_string())
}

async fn blog_handler(
    axum::extract::Path(id): axum::extract::Path<i64>,
    State(state): State<Arc<Shared>>,
) -> Html<String> {
    dbg!(id);
    return Html("blog".to_string());
}

// --------------------------------------------------------
// static files
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

// --------------------------------------------------------
// page router
// --------------------------------------------------------
async fn default_handle(uri: Uri, State(shared): State<Arc<Shared>>) -> impl IntoResponse {
    let route = (match uri.path().to_string().strip_suffix(TEMPLATE_EXTENSION) {
        Some(_) => uri.path().to_string(),
        None => format!("{}/index.html", uri.path().to_string()),
    })
    .trim_matches('/')
    .to_string();

    println!("route: {}", route);

    let tmpl = match shared.templates.get_template(&route) {
        Ok(tmpl) => tmpl,
        Err(_) => {
            println!("template not found: {}", &route);
            return Err((StatusCode::BAD_REQUEST, "unknown page"));
        }
    };

    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/html"),
        (axum::http::header::CONTENT_LANGUAGE, "en"),
    ];

    Ok((headers, tmpl.render(context! { name => "Lorenz" }).unwrap()))
}

// --------------------------------------------------------
// file loader
// --------------------------------------------------------
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
                    .skip(1)
                    .collect::<PathBuf>()
                    .to_string_lossy()
                    .to_string()
                    .replace("index.html", "");
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
            .to_string()
            .replace(format!("{}/", PAGE_DIR).as_str(), "");

        println!("loaded template: {} line: {}", file_name, name);

        env.add_template_owned(name, template)
            .expect("error loading template");
    });

    env
}
