use crate::AuthState;
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use std::{cmp::Ordering, sync::Arc};
use tracing::{debug, error};

pub mod google_auth;

pub fn routes(state: Arc<AuthState>) -> Router {
    Router::new()
        .nest_service("/google/", google_auth::routes(state))
}

// auth middleware
pub async fn request_authorizer(
    State(state): State<Arc<AuthState>>,
    jar: CookieJar,
    req: Request,
    next: Next,
) -> Response {
    match jar.get("session_id") {
        Some(cookie) => {
            let session_id = cookie.value_trimmed();
            match state.sessions.lock() {
                Ok(sessions) => {
                    if let Some((user, exp)) = sessions.get(session_id) {
                        if exp.cmp(&time::OffsetDateTime::now_utc()) != Ordering::Greater {
                            return (StatusCode::FORBIDDEN, "Not Authorized".to_string())
                                .into_response();
                        }
                        debug!("cookie active: {:?}", user);
                    } else {
                        return (StatusCode::FORBIDDEN, "Not Authorized".to_string())
                            .into_response();
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                    return (StatusCode::FORBIDDEN, "Not Authorized".to_string()).into_response();
                }
            }
        }
        None => {
            return (StatusCode::FORBIDDEN, "Not Authorized".to_string()).into_response();
        }
    }
    let response = next.run(req).await;

    response
}
