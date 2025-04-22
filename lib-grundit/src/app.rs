// internal imports
use crate::auth;
pub use crate::auth::google_auth::GoogleAuthClient;
pub use crate::error::AuthrError;
pub use crate::types::ExtractGlonkQueries;
use crate::types::{Comment, Note, Punch, User};
pub use crate::types::{DataType, RequestComment, RequestNote, RequestPunch, RequestUser};

// imports
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::request::Parts;
use axum::middleware;
use axum::response::AppendHeaders;
use axum::{
    Json, Router,
    extract::{Path, State},
    handler::HandlerWithoutStateExt,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use lib_glonk::store::{SqliteStore, Store};
use lib_glonk::types::{DataObject, RequestObject};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};

// state type
pub struct AuthrState {
    pub auth: Arc<AuthState>,
    pub data: Arc<DataState>,
}

pub struct AuthState {
    pub(crate) oauth_sessions: Mutex<HashMap<String, String>>,
    pub(crate) sessions: Mutex<HashMap<String, (User, time::OffsetDateTime)>>,
    pub(crate) google_client: GoogleAuthClient,
    pub(crate) store: Arc<SqliteStore>,
}

pub struct DataState {
    store: Arc<SqliteStore>,
}

impl AuthrState {
    pub fn new(google_client: GoogleAuthClient, store: SqliteStore) -> Self {
        let store = Arc::new(store);
        Self {
            auth: Arc::new(AuthState {
                oauth_sessions: Mutex::new(HashMap::<String, String>::new()),
                sessions: Mutex::new(HashMap::<String, (User, time::OffsetDateTime)>::new()),
                google_client,
                store: store.clone(),
            }),
            data: Arc::new(DataState { store }),
        }
    }
}

async fn data_get_queries(
    Path(data_type): Path<DataType>,
    ExtractGlonkQueries(queries): ExtractGlonkQueries,
    State(state): State<Arc<DataState>>,
) -> impl IntoResponse {
    debug!("extracted queries {:?}", queries);
    match data_type {
        DataType::User => {
            let data = state.store.clone().get_queries::<User>(queries);
            Json(data.clone()).into_response()
        }
        DataType::Note => {
            let data = state.store.clone().get_queries::<Note>(queries);
            Json(data.clone()).into_response()
        }
        DataType::Comment => {
            let data = state.store.clone().get_queries::<Comment>(queries);
            Json(data.clone()).into_response()
        }
        DataType::Punch => {
            let data = state.store.clone().get_queries::<Punch>(queries);
            Json(data.clone()).into_response()
        }
    }
}

async fn data_get(
    Path((data_type, id)): Path<(DataType, i64)>,
    OwnerIdHeader(owner_id): OwnerIdHeader,
    State(state): State<Arc<DataState>>,
) -> impl IntoResponse {
    info!("{:?}", owner_id);
    match data_type {
        DataType::User => {
            let data: Option<User> = state.store.clone().get(id);
            match data {
                Some(data) => Json(data.clone()).into_response(),
                None => AuthrError::NotFound.into_response(),
            }
        }
        DataType::Note => {
            let data: Option<Note> = state.store.clone().get(id);
            match data {
                Some(data) => Json(data.clone()).into_response(),
                None => AuthrError::NotFound.into_response(),
            }
        }
        DataType::Comment => {
            let data: Option<Comment> = state.store.clone().get(id);
            match data {
                Some(data) => Json(data.clone()).into_response(),
                None => AuthrError::NotFound.into_response(),
            }
        }
        DataType::Punch => {
            let data: Option<Punch> = state.store.clone().get(id);
            match data {
                Some(data) => Json(data.clone()).into_response(),
                None => AuthrError::NotFound.into_response(),
            }
        }
    }
}

async fn data_delete(
    Path((data_type, id)): Path<(DataType, i64)>,
    OwnerIdHeader(owner_id): OwnerIdHeader,
    State(state): State<Arc<DataState>>,
) -> impl IntoResponse {
    match data_type {
        DataType::User => {
            let data = state.store.clone().delete::<User>(id, owner_id);
            match data {
                Ok(data) => Json(data.clone()).into_response(),
                Err(_) => AuthrError::NotFound.into_response(),
            }
        }
        DataType::Note => {
            let data = state.store.clone().delete::<Note>(id, owner_id);
            match data {
                Ok(data) => Json(data.clone()).into_response(),
                Err(_) => AuthrError::NotFound.into_response(),
            }
        }
        DataType::Comment => {
            let data = state.store.clone().delete::<Comment>(id, owner_id);
            match data {
                Ok(data) => Json(data.clone()).into_response(),
                Err(_) => AuthrError::NotFound.into_response(),
            }
        }
        DataType::Punch => {
            let data = state.store.clone().delete::<Punch>(id, owner_id);
            match data {
                Ok(data) => Json(data.clone()).into_response(),
                Err(_) => AuthrError::NotFound.into_response(),
            }
        }
    }
}

async fn handle_create<R: RequestObject + Clone, T: DataObject + Serialize>(
    payload: R,
    state: Arc<DataState>,
    owner_id: Option<i64>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate_create(owner_id) {
        error!("{:?}", e);
        return AuthrError::NotFound.into_response();
    }
    let data = state.store.clone().create::<_, T>(payload);
    match data {
        Ok(data) => Json(data.clone()).into_response(),
        Err(_) => AuthrError::NotFound.into_response(),
    }
}

async fn data_create(
    Path(data_type): Path<DataType>,
    OwnerIdHeader(owner_id): OwnerIdHeader,
    State(state): State<Arc<DataState>>,
    body: String,
) -> impl IntoResponse {
    match data_type {
        DataType::User => match serde_json::from_str::<RequestUser>(body.as_str()) {
            Ok(payload) => handle_create::<_, User>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
        DataType::Note => match serde_json::from_str::<RequestNote>(body.as_str()) {
            Ok(payload) => handle_create::<_, Note>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
        DataType::Comment => match serde_json::from_str::<RequestComment>(body.as_str()) {
            Ok(payload) => handle_create::<_, Comment>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
        DataType::Punch => match serde_json::from_str::<RequestPunch>(body.as_str()) {
            Ok(payload) => handle_create::<_, Punch>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
    }
}

async fn handle_update<R: RequestObject + Clone, T: DataObject + Serialize>(
    payload: R,
    state: Arc<DataState>,
    owner_id: Option<i64>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate_update(owner_id) {
        error!("{:?}", e);
        return AuthrError::NotFound.into_response();
    }
    let data = state.store.clone().update::<_, T>(payload);
    match data {
        Ok(data) => Json(data.clone()).into_response(),
        Err(_) => AuthrError::NotFound.into_response(),
    }
}

async fn data_update(
    Path(data_type): Path<DataType>,
    OwnerIdHeader(owner_id): OwnerIdHeader,
    State(state): State<Arc<DataState>>,
    body: String,
) -> impl IntoResponse {
    match data_type {
        DataType::User => match serde_json::from_str::<RequestUser>(body.as_str()) {
            Ok(payload) => handle_update::<_, User>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
        DataType::Note => match serde_json::from_str::<RequestNote>(body.as_str()) {
            Ok(payload) => handle_update::<_, Note>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
        DataType::Comment => match serde_json::from_str::<RequestComment>(body.as_str()) {
            Ok(payload) => handle_update::<_, Comment>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
        DataType::Punch => match serde_json::from_str::<RequestPunch>(body.as_str()) {
            Ok(payload) => handle_update::<_, Punch>(payload, state, owner_id)
                .await
                .into_response(),
            Err(e) => {
                error!("{:?}", e);
                return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
            }
        },
    }
}

async fn whoami(
    OwnerIdHeader(owner_id): OwnerIdHeader,
    State(state): State<Arc<DataState>>,
) -> impl IntoResponse {
    let owner_id = match owner_id {
        Some(oid) => oid,
        None => {
            error!("No owner_id specified for whoami endpoint");
            return AuthrError::NotFound.into_response();
        }
    };
    let data: Option<User> = state.store.clone().get(owner_id);
    match data {
        Some(data) => Json(data.clone()).into_response(),
        None => AuthrError::NotFound.into_response(),
    }
}

// helper functions
async fn handle_not_found() -> impl IntoResponse {
    AuthrError::NotFound.into_response()
}

fn data_routes(state: Arc<DataState>) -> Router {
    Router::new()
        .route("/{type}/{id}", get(data_get))
        .route("/{type}", get(data_get_queries))
        .route("/{type}/{id}", delete(data_delete))
        .route("/{type}", post(data_create))
        .route("/{type}", put(data_update))
        .route("/whoami", get(whoami))
        .with_state(state)
}

pub async fn logout(State(state): State<Arc<AuthState>>, jar: CookieJar) -> impl IntoResponse {
    // get session id
    let session_id = match jar.get("session_id") {
        Some(cookie) => cookie.value_trimmed(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing session_id").into_response();
        }
    };
    // invalidate session cache
    match state.sessions.lock() {
        Ok(mut sessions) => {
            if let Some((user, _)) = sessions.remove(session_id) {
                debug!("Logging out {:?}", user);
            } else {
                error!(
                    "invalid session id made it through auth phase: {}",
                    session_id
                );
            }
        }
        Err(e) => {
            // report the error and invalidate the cookie
            error!("{:?}", e);
        }
    }
    // revoke cookie
    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .http_only(true)
        .build();

    (
        StatusCode::TEMPORARY_REDIRECT,
        AppendHeaders([(SET_COOKIE, cookie.to_string().as_str()), (LOCATION, "/")]),
    )
        .into_response()
}

pub async fn run(listener: TcpListener, state: AuthrState) {
    let state = Arc::new(state);
    let app = Router::new()
        // routes behind auth
        // data
        .nest_service("/data/", data_routes(state.data.clone()))
        // static files
        .nest_service(
            "/web",
            ServeDir::new("./splunge/home").not_found_service(handle_not_found.into_service()),
        )
        // logout gets auth state
        .route("/auth/logout", get(logout))
        .with_state(state.auth.clone())
        // auth layer
        .route_layer(middleware::from_fn_with_state(
            state.auth.clone(),
            auth::request_authorizer,
        ))
        // no auth routes (login, etc.)
        .nest_service("/auth/", auth::routes(state.auth.clone()))
        // fallback to static
        .fallback_service(
            ServeDir::new("./splunge/auth").not_found_service(handle_not_found.into_service()),
        );

    info!("Listening on {:?}", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}

pub struct OwnerIdHeader(Option<i64>);

impl<S> FromRequestParts<S> for OwnerIdHeader
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get("Owner-Id") {
            Some(val) => {
                if let Ok(v_str) = val.to_str() {
                    match v_str.parse::<i64>() {
                        Ok(v) => Ok(OwnerIdHeader(Some(v))),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(())
                        }
                    }
                } else {
                    Err(())
                }
            }
            None => Ok(OwnerIdHeader(None)),
        }
    }
}
