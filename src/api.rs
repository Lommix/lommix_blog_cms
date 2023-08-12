use crate::auth::Auth;
use crate::store::articles::Article;
use crate::store::paragraphs::Paragraph;
use crate::store::paragraphs::ParagraphType;

use super::store::*;
use super::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::Form;
use axum::Router;
use minijinja::context;
use std::sync::Arc;

macro_rules! is_admin {
    ($state:expr) => {
        if !$state.is_admin() {
            return Err((
                StatusCode::NETWORK_AUTHENTICATION_REQUIRED,
                "not authorized",
            ));
        }
    };
}

pub fn api_routes() -> Router<Arc<SharedState>, axum::body::Body> {
    Router::new()
        .route("/article", post(article_create).get(article_list))
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
        .route("/paragraph/parsed/:id", get(paragraph_parsed))
        .route("/file_list", get(file_list))
}

// ------------------------------------------------------
// articles
// ------------------------------------------------------
pub async fn article_list(State(state): State<Arc<SharedState>>, auth: Auth) -> impl IntoResponse {
    let articles = Article::find_all(&state.db).expect("failed to find all articles");
    let tmpl = state
        .templates
        .get_template("components/article_preview_box.html")
        .unwrap();

    Html(
        tmpl.render(context! {
            articles => articles,
            auth => auth
        })
        .unwrap(),
    )
}

async fn article_delete(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
    auth: Auth,
) -> impl IntoResponse {
    is_admin!(auth);
    match Article::delete(id, &state.db) {
        Ok(_) => Ok((StatusCode::OK, Html("deleted".to_string()))),
        Err(e) => Err((StatusCode::BAD_REQUEST, "failed to delete")),
    }
}

async fn article_get(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    let article = Article::find(id, &state.db).unwrap();
    Html("blog_detail".to_string())
}

#[derive(serde::Deserialize)]
pub struct ArticleForm {
    title: String,
}

pub async fn article_create(
    auth: Auth,
    State(state): State<Arc<SharedState>>,
    Form(form): Form<ArticleForm>,
) -> impl IntoResponse {
    is_admin!(auth);
    let mut article = Article::new(form.title);
    match article.insert(&state.db) {
        Ok(_) => Ok((StatusCode::CREATED, Html("created".to_string()))),
        Err(e) => Err((StatusCode::BAD_REQUEST, "failed to create")),
    }
}

async fn article_update(
    auth: Auth,
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
    Form(form): Form<ArticleForm>,
) -> impl IntoResponse {
    is_admin!(auth);
    Ok(Html("blog_update".to_string()))
}

// ------------------------------------------------------
// paragraphs
// ------------------------------------------------------
async fn paragraph_get(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    let paragraph = Paragraph::find(id, &state.db).unwrap();
    Html("paragraph_detail".to_string())
}

#[derive(serde::Deserialize)]
struct ParagraphForm {
    id: Option<i64>,
    article_id: i64,
    paragraph_type: ParagraphType,
    content: String,
}

async fn paragraph_update(
    auth: Auth,
    State(state): State<Arc<SharedState>>,
    Form(form): Form<ParagraphForm>,
) -> impl IntoResponse {
    is_admin!(auth);

    let id = match form.id {
        Some(id) => id,
        None => return Err((StatusCode::BAD_REQUEST, "missing id")),
    };

    let mut paragraph = match Paragraph::find(id, &state.db) {
        Ok(p) => p,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "not found")),
    };

    paragraph.content = form.content;
    paragraph.paragraph_type = form.paragraph_type;

    match paragraph.update(&state.db) {
        Ok(_) => Ok((StatusCode::OK, Html("updated".to_string()))),
        Err(_) => Err((StatusCode::BAD_REQUEST, "failed to update")),
    }
}

async fn paragraph_create(
    auth: Auth,
    State(state): State<Arc<SharedState>>,
    Form(form): Form<ParagraphForm>,
) -> impl IntoResponse {
    is_admin!(auth);

    let paragraph = Paragraph {
        id: form.id,
        article_id: form.article_id,
        paragraph_type: form.paragraph_type,
        content: form.content,
        position: 0,
        title: "".to_string(),
        description: "".to_string(),
    }
    .insert(&state.db);

    match paragraph {
        Ok(p) => Ok((StatusCode::CREATED, Html("created".to_string()))),
        Err(_) => Err((StatusCode::BAD_REQUEST, "failed to create")),
    }
}

async fn paragraph_parsed(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    match Paragraph::get_parsed(id, &state.db) {
        Ok(p) => Ok((StatusCode::OK, Html(p.to_string()))),
        Err(_) => Err((StatusCode::BAD_REQUEST, Html("not found".to_string()))),
    }
}

async fn paragraph_delete(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
    auth: Auth,
) -> impl IntoResponse {
    is_admin!(auth);
    match Paragraph::delete(id, &state.db) {
        Ok(_) => Ok((StatusCode::OK, Html("deleted".to_string()))),
        Err(e) => Err((StatusCode::BAD_REQUEST, "failed to delete")),
    }
}
// ------------------------------------------------------
// files
// ------------------------------------------------------
async fn file_list(auth: Auth) -> impl IntoResponse {
    is_admin!(auth);

    let file_list = match crate::util::Util::load_files_rec("static/media".into()) {
        Ok(file_list) => file_list,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "failed to delete")),
    };

    let file_string = file_list
        .iter()
        .map(|(_, path)| path.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n")
        .to_string();

    Ok(Html(file_string))
}

async fn file_upload(auth: Auth) -> impl IntoResponse {
    is_admin!(auth);

    Ok(Html("file_upload".to_string()))
}
