#![allow(unused)]

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

mod api;
mod pages;

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

    let app = Router::new().route("/", get(root)).with_state(state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn load_files() -> Vec<(String, PathBuf)> {
    vec![]
}

fn load_templates() -> minijinja::Environment<'static> {
    let templates = load_files();
    let mut env = minijinja::Environment::new();
    env.add_template("hello", "Hello {{name}}").unwrap();
    env
}

async fn root(State(shared): State<Arc<Shared<'_>>>) -> String {
    let tmpx = shared.templates.get_template("hello").unwrap();
    tmpx.render(context! { name => "Lorenz" }).unwrap()
}
