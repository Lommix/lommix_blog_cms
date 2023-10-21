use crate::auth::Auth;
use crate::auth::AUTH_COOKIE;
use crate::store::articles::Article;
use crate::store::contacts::ContactRequest;
use crate::store::paragraphs::Paragraph;
use crate::store::paragraphs::ParagraphType;
use crate::store::stats::Stats;
use crate::Session;
use crate::UserState;

use super::store::*;
use super::SharedState;
use axum::extract::Query;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::Form;
use axum::Router;
use minijinja::context;
use std::path::PathBuf;
use std::sync::Arc;

const ADMIN_USER: &str = "ADMIN_USER";
const ADMIN_PASSWORD: &str = "ADMIN_PASSWORD";

macro_rules! require_admin {
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
        .route("/articles/:offset/:limit", get(article_list_paginated))
        .route("/articles/:offset/:limit/:tag", get(article_list_paginated_filterd))
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
        .route("/files", get(file_list))
        .route("/files/:id", post(file_upload))
        .route("/login", post(login))
        .route("/stats", get(get_stats))
        .route("/logout", get(logout))
        .route("/contact", post(post_contact))
        .route("/contact/:id", get(get_contact_message))
}

// ------------------------------------------------------
// login
// ------------------------------------------------------

#[derive(serde::Deserialize)]
struct LoginForm {
    user: String,
    password: String,
}

async fn login(
    auth: Auth,
    State(state): State<Arc<SharedState>>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let user = std::env::var(ADMIN_USER).unwrap();
    let pw = std::env::var(ADMIN_PASSWORD).unwrap();

    if form.user == user && form.password == pw {
        let cookie_hash = rand::random::<u128>();

        if let Ok(mut sessions) = state.sessions.write() {
            sessions.push(Session {
                id: cookie_hash,
                user_state: UserState::Admin,
            });
        }

        let mut header = HeaderMap::new();

        header.insert(
            "set-cookie",
            format!("{}={}; Path=/", AUTH_COOKIE, cookie_hash)
                .parse()
                .unwrap(),
        );
        header.insert("HX-Refresh", "true".parse().unwrap());

        return Ok((header, Html("success".to_string())));
    }
    Err((StatusCode::OK, "failed to login, fail again and are banned"))
}

// ------------------------------------------------------
// logout
// ------------------------------------------------------
async fn logout(auth: Auth, State(state): State<Arc<SharedState>>) -> impl IntoResponse {
    require_admin!(auth);

    if let Ok(mut sessions) = state.sessions.write() {
        sessions.retain(|s| s.id != auth.id.unwrap());
    }

    let mut header = HeaderMap::new();
    header.insert(
        "set-cookie",
        format!(
            "{}=; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/",
            AUTH_COOKIE
        )
        .parse()
        .unwrap(),
    );
    header.insert("HX-Redirect", "/".parse().unwrap());

    Ok((header, Html("success".to_string())))
}

// ------------------------------------------------------
// articles
// ------------------------------------------------------
pub async fn article_list(State(state): State<Arc<SharedState>>, auth: Auth) -> impl IntoResponse {
    let mut articles = match Article::find_all(&state.db) {
        Ok(articles) => articles,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "failed to find articles")),
    };

    let tmpl = match state
        .templates
        .get_template("components/article_preview_box.html")
    {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template")),
    };

    if !auth.is_admin() {
        articles = articles
            .drain(..)
            .filter(|a| a.published)
            .collect::<Vec<_>>();
    }

    Ok(Html(
        tmpl.render(context! {
            articles => articles,
            auth => auth
        })
        .unwrap(),
    ))
}

pub async fn article_list_paginated_filterd(
    Path((offset, limit, tag)): Path<(i64, i64, String)>,
    State(state): State<Arc<SharedState>>,
    auth: Auth,
) -> Response {
    let mut articles = match Article::find_articles_paginated(&state.db, &tag, offset, limit) {
        Ok(articles) => articles,
        Err(err) => {
            dbg!(err);
            return (StatusCode::BAD_REQUEST, "failed to find articles").into_response()},
    };

    let tmpl = match state
        .templates
        .get_template("components/article_preview_box.html")
    {
        Ok(tmpl) => tmpl,
        Err(_) => return (StatusCode::BAD_REQUEST, "missing template").into_response(),
    };

    if !auth.is_admin() {
        articles = articles
            .drain(..)
            .filter(|a| a.published)
            .collect::<Vec<_>>();
    }

    Html(
        tmpl.render(context! {
            articles => articles,
            auth => auth
        })
        .unwrap(),
    )
    .into_response()
}

pub async fn article_list_paginated(
    Path((offset, limit)): Path<(i64, i64)>,
    State(state): State<Arc<SharedState>>,
    auth: Auth,
) -> Response {
    let mut articles = match Article::find_articles_paginated(&state.db, "", offset, limit) {
        Ok(articles) => articles,
        Err(_) => return (StatusCode::BAD_REQUEST, "failed to find articles").into_response(),
    };
    let tmpl = match state
        .templates
        .get_template("components/article_preview_box.html")
    {
        Ok(tmpl) => tmpl,
        Err(_) => return (StatusCode::BAD_REQUEST, "missing template").into_response(),
    };

    if !auth.is_admin() {
        articles = articles
            .drain(..)
            .filter(|a| a.published)
            .collect::<Vec<_>>();
    }

    Html(
        tmpl.render(context! {
            articles => articles,
            auth => auth
        })
        .unwrap(),
    )
    .into_response()
}

async fn article_delete(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
    auth: Auth,
) -> impl IntoResponse {
    require_admin!(auth);
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
    teaser: Option<String>,
    cover: Option<String>,
    alias: Option<String>,
    published: bool,
}

pub async fn article_create(
    auth: Auth,
    State(state): State<Arc<SharedState>>,
    Form(form): Form<ArticleForm>,
) -> impl IntoResponse {
    require_admin!(auth);
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
    require_admin!(auth);

    let mut article = match Article::find(id, &state.db) {
        Ok(a) => a,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "not found")),
    };

    article.title = form.title;
    article.teaser = form.teaser.unwrap_or("".to_string());
    article.cover = form.cover.unwrap_or("".to_string());
    article.alias = form.alias.unwrap_or("".to_string());
    article.published = form.published;

    let tmpl = match state
        .templates
        .get_template("components/article_header.html")
    {
        Ok(t) => t,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get template")),
    };

    let html = match tmpl.render(context! {
        article => article
    }) {
        Ok(h) => h,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to render template",
            ))
        }
    };

    match article.update(&state.db) {
        Ok(_) => Ok((StatusCode::OK, Html(html))),
        Err(_) => Err((StatusCode::BAD_REQUEST, "failed to update")),
    }
}

// ------------------------------------------------------
// paragraphs
// ------------------------------------------------------
async fn paragraph_get(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    match Paragraph::find(id, &state.db) {
        Ok(p) => {
            let rendered = match p.rendered {
                Some(r) => r,
                None => p.content,
            };
            Ok((StatusCode::OK, Html(rendered)))
        }
        Err(_) => Err((StatusCode::BAD_REQUEST, "failed to get")),
    }
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
    require_admin!(auth);

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
    require_admin!(auth);

    let paragraph = Paragraph {
        id: form.id,
        article_id: form.article_id,
        paragraph_type: form.paragraph_type,
        content: form.content,
        position: 0,
        title: "".to_string(),
        description: "".to_string(),
        rendered: None,
    }
    .insert(&state.db);

    match paragraph {
        Ok(p) => Ok((StatusCode::CREATED, Html("created".to_string()))),
        Err(_) => Err((StatusCode::BAD_REQUEST, "failed to create")),
    }
}

async fn paragraph_delete(
    Path(id): Path<i64>,
    State(state): State<Arc<SharedState>>,
    auth: Auth,
) -> impl IntoResponse {
    require_admin!(auth);
    match Paragraph::delete(id, &state.db) {
        Ok(_) => Ok((StatusCode::OK, Html("deleted".to_string()))),
        Err(e) => Err((StatusCode::BAD_REQUEST, "failed to delete")),
    }
}

// ------------------------------------------------------
// files
// ------------------------------------------------------
async fn file_list(auth: Auth) -> impl IntoResponse {
    require_admin!(auth);

    let file_list = match crate::util::Util::load_files_rec("static/media".into()) {
        Ok(file_list) => file_list,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "failed to delete")),
    };

    let file_string = file_list
        .iter()
        .map(|(_, path)| path.to_str().unwrap().to_string())
        .map(|path| format!("<option value=/{} />", path))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(Html(file_string))
}

async fn file_upload(
    Path(id): Path<i64>,
    auth: Auth,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    require_admin!(auth);
    // todo safe the unsafe
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let folder_path: PathBuf = format!("static/media/{}/", id).into();
        std::fs::create_dir_all(&folder_path).unwrap();
        let file_path = folder_path.join(name);
        std::fs::write(file_path, data).unwrap();
    }

    Ok(Html("file_upload".to_string()))
}

// ------------------------------------------------------
// stats
// ------------------------------------------------------
async fn get_stats(auth: Auth, State(state): State<Arc<SharedState>>) -> impl IntoResponse {
    require_admin!(auth);

    let stats = match Stats::get_last_days(3, &state.db) {
        Ok(stats) => stats,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "failed to get")),
    };

    let message_count = match ContactRequest::count_all(&state.db) {
        Ok(count) => count,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "failed to get")),
    };

    let top_ten = match ContactRequest::find_all_orderd(10, 0, &state.db) {
        Ok(top_ten) => top_ten,
        Err(e) => {
            println!("error: {}", e);
            return Err((StatusCode::BAD_REQUEST, "failed to get"));
        }
    };

    let tmpl = match state.templates.get_template("components/stats.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template")),
    };

    Ok(Html(
        tmpl.render(context! {stats=>stats, message_count=>message_count, top_ten=>top_ten})
            .unwrap(),
    ))
}

// ------------------------------------------------------
// Contact requests
// ------------------------------------------------------
#[derive(serde::Deserialize)]
struct ContactForm {
    email: String,
    subject: String,
    message: String,
}

async fn post_contact(
    State(state): State<Arc<SharedState>>,
    Form(form): Form<ContactForm>,
) -> impl IntoResponse {
    let mut contact_request = ContactRequest {
        id: None,
        created: chrono::offset::Local::now().timestamp(),
        email: form.email,
        subject: form.subject,
        message: form.message,
    };

    contact_request.insert(&state.db);

    let tmpl = match state.templates.get_template("components/success.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template")),
    };

    Ok(Html(tmpl.render(context! {}).unwrap()))
}

// ------------------------------------------------------
// Contact message get
// ------------------------------------------------------
async fn get_contact_message(
    Path(id): Path<i64>,
    auth: Auth,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    require_admin!(auth);

    let message = match ContactRequest::find(id, &state.db) {
        Ok(count) => count,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "failed to get")),
    };
    let tmpl = match state.templates.get_template("components/mail.html") {
        Ok(tmpl) => tmpl,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "missing template")),
    };

    Ok(Html(tmpl.render(context! { mail=>message }).unwrap()))
}
