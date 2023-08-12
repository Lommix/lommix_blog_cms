use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts},
    http::{self, request::Parts, Request},
};
use serde::{Deserialize, Serialize};

use crate::{store::articles::Article, store::Crud, SharedState, UserState};

#[derive(Deserialize, Serialize)]
pub struct Auth {
    pub user_state: UserState,
}

impl Auth {
    pub fn is_admin(&self) -> bool {
        self.user_state == UserState::Admin
    }
}

#[async_trait]
impl FromRequestParts<Arc<SharedState>> for Auth {
    type Rejection = http::StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<SharedState>,
    ) -> Result<Self, Self::Rejection> {

        let cookies = parts.headers.get(http::header::COOKIE);
        println!("cookies: {:?}", cookies);
        // do something later
        Ok(Auth {
            user_state: UserState::Admin,
        })
    }
}
