use crate::app::AuthState;
use axum::{
    extract::{Request, State}, http::{HeaderName, HeaderValue, StatusCode}, middleware::Next, response::{IntoResponse, Response}, Router
};
use axum_extra::extract::CookieJar;
use std::{cmp::Ordering, str::FromStr, sync::Arc};
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
    mut req: Request,
    next: Next,
) -> Response {
    let user = match jar.get("session_id") {
        Some(cookie) => {
            let session_id = cookie.value_trimmed();
            match state.sessions.lock() {
                Ok(sessions) => {
                    if let Some((user, exp)) = sessions.get(session_id) {
                        if exp.cmp(&time::OffsetDateTime::now_utc()) != Ordering::Greater {
                            return (StatusCode::FORBIDDEN, "Not Authorized".to_string())
                                .into_response();
                        }
                        user.clone()
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
    };

    debug!("{:?}", user);

    let owner_id = format!("{}", user.id);
    let header_val = match HeaderValue::from_str(owner_id.as_str()) {
        Ok(v) => v,
        Err(e) => {
            error!("{:?}", e);
            return (StatusCode::FORBIDDEN, "Not Authorized".to_string()).into_response();
        }
    };

    let header_name = match HeaderName::from_str("Owner-Id") {
        Ok(v) => v,
        Err(e) => {
            error!("{:?}", e);
            return (StatusCode::FORBIDDEN, "Not Authorized".to_string()).into_response();
        }
    };
    req.headers_mut().insert(header_name, header_val);

    let response = next.run(req).await;

    response
}
