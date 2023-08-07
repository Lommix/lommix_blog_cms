#![allow(unused)]

use axum::{
    body::StreamBody,
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::{
    io::BufReader,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

mod api;
mod pages;
mod store;

const TEMPLATE_DIR: &str = "templates";
const PAGE_DIR: &str = "pages";

// --------------------------------------------------------
// shared state
// --------------------------------------------------------

struct Shared<'source> {
    pub db: rusqlite::Connection,
    pub templates: minijinja::Environment<'source>,
}

unsafe impl<'source> Send for Shared<'source> {}
unsafe impl<'source> Sync for Shared<'source> {}

// --------------------------------------------------------
// main
// --------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let state = Arc::new(Shared {
        db: rusqlite::Connection::open_in_memory().unwrap(),
        templates: load_templates(),
    });

    println!("listening on {}", addr);

    let app = Router::new()
        .route("/static/*asset", get(asset))
        .route("/*url", get(root))
        .route("/", get(root))
        .with_state(state)
        .into_make_service();

    axum::Server::bind(&addr).serve(app).await.unwrap();
}

async fn asset(uri: Uri) -> impl IntoResponse {
    // add code here
    let p: PathBuf = uri.path().into();
    let path = p.iter().skip(1).collect::<PathBuf>();

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
            return Err((StatusCode::BAD_REQUEST, "error getting mime"));
        }
    };

    println!("Serving Asset: {}", path.to_str().unwrap());

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let headers = [(axum::http::header::CONTENT_TYPE, content_type)];

    Ok((headers, body))
}

async fn root(uri: Uri, State(shared): State<Arc<Shared<'_>>>) -> impl IntoResponse {
    let route = (match uri.path().to_string().strip_suffix(".html") {
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

fn load_dir(dir: PathBuf) -> Vec<(String, PathBuf)> {
    let mut files = Vec::new();
    std::fs::read_dir(dir)
        .expect("unable to load dir")
        .for_each(|file| {
            let path = file.unwrap().path();
            if path.is_dir() {
                files.append(&mut load_dir(path));
            } else if path.is_file() && path.extension() == Some("html".as_ref()) {
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
