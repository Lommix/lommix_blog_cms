use std::sync::Arc;

use crate::{store::articles::Article, store::{Crud, stats::Stats}, SharedState, UserState};
use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts},
    http::{self, request::Parts, HeaderValue, Request},
};
use serde::{Deserialize, Serialize};

pub const AUTH_COOKIE: &str = "auth";

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Auth {
    pub id: Option<u128>,
    pub user_state: UserState,
}

impl Auth {
    pub fn new(id: u128, user_state: UserState) -> Self {
        Auth {
            id: Some(id),
            user_state,
        }
    }
    pub fn is_admin(&self) -> bool {
        self.user_state == UserState::Admin
    }

    pub fn is_user(&self) -> bool {
        self.user_state == UserState::User
    }
}

#[derive(Debug)]
pub struct CookieJar {
    pub cookies: Vec<(String, String)>,
}

impl CookieJar {
    pub fn try_get(&self, name: &str) -> Option<&str> {
        for (k, v) in &self.cookies {
            if k == name {
                return Some(v);
            }
        }
        None
    }
}

impl TryFrom<&HeaderValue> for CookieJar {
    type Error = ();

    fn try_from(header: &HeaderValue) -> Result<Self, Self::Error> {
        let cookies = header
            .to_str()
            .map_err(|_| ())?
            .split(';')
            .map(|s| {
                let pair = s.split('=').collect::<Vec<_>>();

                if pair.len() != 2 {
                    return Err(());
                }

                Ok((pair[0].to_string(), pair[1].to_string()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ())?;

        Ok(CookieJar { cookies })
    }
}

impl CookieJar {}

#[async_trait]
impl FromRequestParts<Arc<SharedState>> for Auth {
    type Rejection = http::StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<SharedState>,
    ) -> Result<Self, Self::Rejection> {

        let cookie_value = match parts.headers.get("cookie") {
            Some(c) => c,
            None => return Ok(Auth::default()),
        };

        let cookies = match CookieJar::try_from(cookie_value) {
            Ok(c) => c,
            Err(_) => return Ok(Auth::default()),
        };

        match cookies.try_get(AUTH_COOKIE) {
            Some(cookie) => match state.sessions.read() {
                Ok(sessions) => {
                    let cookie_id = match cookie.parse::<u128>() {
                        Ok(id) => id,
                        Err(_) => return Ok(Auth::default()),
                    };
                    match sessions.iter().find(|s| s.id == cookie_id) {
                        Some(s) => Ok(Auth::new(s.id, s.user_state.clone())),
                        None => Ok(Auth::default()),
                    }
                }
                Err(_) => Ok(Auth::default()),
            },
            None => Ok(Auth::default()),
        }
    }
}
