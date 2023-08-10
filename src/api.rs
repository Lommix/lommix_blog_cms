use crate::store::articles::Article;
use crate::store::paragraphs::Paragraph;

use super::store::*;
use super::Shared;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::Form;
use axum::Router;
use std::sync::Arc;

pub fn api_routes() -> Router<Arc<Shared>, axum::body::Body> {
    Router::new()
        .route("/article/create", post(article_create))
        .route(
            "/article/:id",
            get(article_get).delete(article_delete).put(article_update),
        )
        .route("/paragraph", post(paragraph_create))
        .route(
            "/paragraph/:id",
            get(paragraph_get)
                .delete(paragraph_delete)
                .put(paragraph_update),
        )
        .route("/init", get(init_blog))
}

pub async fn init_blog(State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Article::up(&state.db).unwrap();
    Paragraph::up(&state.db).unwrap();
    Html("created database".to_string());
}

// ---------------------------
// List
// ---------------------------
pub async fn article_list(State(state): State<Arc<Shared>>) -> impl IntoResponse {
    let articles = Article::find_all(&state.db).unwrap();
    Html(format!("{:?}", articles))
}
// ---------------------------
// Detail
// ---------------------------
async fn article_get(Path(id): Path<i64>, State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("blog_detail".to_string())
}

async fn paragraph_get(Path(id): Path<i64>, State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("paragraph_detail".to_string())
}
// ---------------------------
// Create
// ---------------------------
#[derive(serde::Deserialize, serde::Serialize)]
struct ArticleForm {
    title: String,
    teaser: String,
    description: String,
}

pub async fn article_create(
    Form(form): Form<ArticleForm>,
    State(state): State<Arc<Shared>>,
) -> impl IntoResponse {
    Html(format!("article_create {}", form.title))
}
async fn paragraph_create(State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("paragraph_create".to_string())
}
// ---------------------------
// Delete
// ----------------------------
async fn article_delete(
    Path(id): Path<i64>,
    State(state): State<Arc<Shared>>,
) -> impl IntoResponse {
    Html("blog_delete".to_string())
}
async fn paragraph_delete(
    Path(id): Path<i64>,
    State(state): State<Arc<Shared>>,
) -> impl IntoResponse {
    Html("Paragraph_delete".to_string())
}

// ---------------------------
// Update
// ----------------------------
async fn article_update(
    Path(id): Path<i64>,
    State(state): State<Arc<Shared>>,
) -> impl IntoResponse {
    Html("blog_update".to_string())
}
async fn paragraph_update(
    Path(id): Path<i64>,
    State(state): State<Arc<Shared>>,
) -> impl IntoResponse {
    Html("paragraph_update".to_string())
}
