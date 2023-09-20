use crate::store::articles::Article;
use crate::store::paragraphs::Paragraph;

use super::auth::Auth;
use super::store::*;
use super::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::Form;
use axum::Router;
use minijinja::context;
use minijinja::Template;
use std::sync::Arc;
use tokio_util::either::Either;

pub fn page_routes() -> Router<Arc<SharedState>, axum::body::Body> {
    Router::new()
        .route("/", get(get_home))
        .route("/about", get(get_about))
        .route("/article/:alias", get(get_article_detail))
        .route("/donate", get(get_donate))
}

// ----------------------------------------
// home
// lommix.de/
// ----------------------------------------
async fn get_home(State(state): State<Arc<SharedState>>, auth: Auth) -> impl IntoResponse {
    let tmpl = match state.templates.get_template("pages/home.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template".to_string())),
    };

    let rendered = match tmpl.render(context! {
        auth => auth,
    }) {
        Ok(html) => html,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "fucked up template".to_string())),
    };

    Ok(Html(rendered))
}
// ----------------------------------------
// about
// lommix.de/about
// ----------------------------------------
async fn get_about(State(state): State<Arc<SharedState>>, auth: Auth) -> impl IntoResponse {
    let tmpl = match state.templates.get_template("pages/about.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template".to_string())),
    };

    let rendered = match tmpl.render(context! {
        auth => auth,
    }) {
        Ok(html) => html,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "fucked up template".to_string())),
    };

    Ok(Html(rendered))
}

// ----------------------------------------
// about
// lommix.de/donate
// ----------------------------------------
async fn get_donate(State(state): State<Arc<SharedState>>, auth: Auth) -> impl IntoResponse {
    let tmpl = match state.templates.get_template("pages/donate.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template".to_string())),
    };

    let rendered = match tmpl.render(context! {
        auth => auth,
    }) {
        Ok(html) => html,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "fucked up template".to_string())),
    };

    Ok(Html(rendered))
}

// ----------------------------------------
// home
// lommix.de/article/:alias
// ----------------------------------------
async fn get_article_detail(
    Path(alias): Path<String>,
    auth: Auth,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {


    //parse alias to int
    let result = match alias.parse::<i64>() {
        Ok(x) => Article::find(x, &state.db),
        Err(_) => Article::find_by_alias(&alias, &state.db),
    };

    let article = match result {
        Ok(article) => article,
        Err(_) => return Err((StatusCode::NOT_FOUND, "not found".to_string())),
    };

    if (!article.published && !auth.is_admin()) {
        return Err((StatusCode::NOT_FOUND, "not found".to_string()));
    }

    let tmpl = match state.templates.get_template("pages/article.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template".to_string())),
    };

    let rendered = match tmpl.render(context! {
        auth => auth,
        article => article,
    }) {
        Ok(html) => html,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "fucked up template".to_string())),
    };

    Ok(Html(rendered))
}

// ----------------------------------------
// home
// lommix.de/article/:id
// ----------------------------------------
async fn get_article_detail_from_id(
    Path(id): Path<i64>,
    auth: Auth,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {

    let article = match Article::find(id, &state.db) {
        Ok(article) => article,
        Err(_) => return Err((StatusCode::NOT_FOUND, "not found".to_string())),
    };

    if (!article.published && !auth.is_admin()) {
        return Err((StatusCode::NOT_FOUND, "not found".to_string()));
    }

    let tmpl = match state.templates.get_template("pages/article.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template".to_string())),
    };

    let rendered = match tmpl.render(context! {
        auth => auth,
        article => article,
    }) {
        Ok(html) => html,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "fucked up template".to_string())),
    };

    Ok(Html(rendered))
}
