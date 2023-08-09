use super::Shared;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::Router;
use std::sync::Arc;

pub fn api_routes() -> Router<Arc<Shared>, axum::body::Body> {
    Router::new()
        .route(
            "/blog/:id",
            get(blog_detail).delete(blog_delete).put(blog_update),
        )
        .route("/blog", get(blog_list))
        .route("/blog/create", post(blog_create))
}

// ---------------------------
// List
// ---------------------------
pub async fn blog_list(State(state): State<Arc<Shared>>) -> impl IntoResponse {
    if false {
        return Err((StatusCode::BAD_REQUEST, "unknown page"));
    }

    Ok(Html("blog_list".to_string()))
}

// ---------------------------
// Detail
// ---------------------------
async fn blog_detail(Path(id): Path<i64>, State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("blog_detail".to_string())
}

// ---------------------------
// Create
// ---------------------------
async fn blog_create(State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("blog_create".to_string())
}

// ---------------------------
// Delete
// ----------------------------
async fn blog_delete(Path(id): Path<i64>, State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("blog_delete".to_string())
}

// ---------------------------
// Update
// ----------------------------
async fn blog_update(Path(id): Path<i64>, State(state): State<Arc<Shared>>) -> impl IntoResponse {
    Html("blog_update".to_string())
}
