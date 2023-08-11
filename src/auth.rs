use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequest,
    http::{self, Request},
};
use serde::{Deserialize, Serialize};

use crate::{store::articles::Article, store::Crud, SharedState, UserState};

#[derive(Deserialize, Serialize)]
pub struct Auth {
    pub user_state: UserState,
}

impl Auth{
    pub fn is_admin(&self) -> bool {
        self.user_state == UserState::Admin
    }
}

#[async_trait]
impl<B> FromRequest<Arc<SharedState>, B> for Auth
where
    B: Send + 'static,
{
    type Rejection = http::StatusCode;
    async fn from_request(req: Request<B>, state: &Arc<SharedState>) -> Result<Self, Self::Rejection> {
        // do something later
        Ok(Auth {
            user_state: UserState::Admin,
        })
    }
}
