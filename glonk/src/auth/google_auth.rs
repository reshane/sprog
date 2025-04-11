use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    extract::{Query, State},
    http::{
        StatusCode,
        header::{LOCATION, SET_COOKIE},
    },
    response::{self, AppendHeaders, IntoResponse},
    routing::get,
};
use axum_extra::extract::cookie::Cookie;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient, reqwest,
};
use oauth2::{
    Client, StandardRevocableToken,
    basic::{
        BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
        BasicTokenResponse,
    },
};
use std::env;

use crate::{
    AuthState, Store,
    error::AuthrError,
    types::{QueryTypes, RequestUser, User, UserByGuid, UserQuery},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

// there has to be a way to get rid of this
// type SetClient<
pub type SetClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointSet,
> = Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
>;

#[derive(Debug)]
pub struct GoogleAuthClient {
    pub client: SetClient,
}

impl GoogleAuthClient {
    pub fn from_env() -> Self {
        let client_id = env::var("GOOGLE_OAUTH_CLIENT_ID").expect("client id");
        let client_secret = env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("client secret");
        let auth_uri = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("auth_uri");
        let token_uri =
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).expect("token_uri");
        let redirect_uri =
            RedirectUrl::new("http://localhost:8080/auth/google/callback".to_string())
                .expect("redirect_uri");

        Self {
            client: BasicClient::new(ClientId::new(client_id))
                .set_client_secret(ClientSecret::new(client_secret))
                .set_auth_uri(auth_uri)
                .set_token_uri(token_uri)
                .set_redirect_uri(redirect_uri),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    verified_email: bool,
    name: String,
    given_name: String,
    family_name: String,
    picture: String,
}

impl From<GoogleUserInfo> for RequestUser {
    fn from(value: GoogleUserInfo) -> Self {
        Self {
            id: None,
            guid: Some(format!("google/{}", value.id)),
            email: Some(value.email),
            name: Some(value.name),
            picture: Some(value.picture),
        }
    }
}

// routes
pub fn routes(state: Arc<AuthState>) -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .with_state(state)
}

pub async fn login(State(state): State<Arc<AuthState>>) -> impl IntoResponse {
    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = state
        .google_client
        .client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    match state.oauth_sessions.lock() {
        Ok(mut oauth_sessions) => {
            oauth_sessions.insert(csrf_token.into_secret(), pkce_verifier.secret().clone());
        }
        Err(e) => {
            error!("{:?}", e);
            return response::Redirect::temporary("/").into_response();
        }
    };

    response::Redirect::temporary(auth_url.as_str()).into_response()
}

pub async fn callback(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AuthState>>,
) -> impl IntoResponse {
    let csrf_token_header = params.get("state");
    let token = match csrf_token_header {
        Some(token) => token,
        None => {
            return AuthrError::NotAuthorized.into_response();
        }
    };

    let code_header = params.get("code");
    let code = match code_header {
        Some(code) => code.to_string(),
        None => {
            return AuthrError::NotAuthorized.into_response();
        }
    };

    let pkce_verifier = match state.oauth_sessions.lock() {
        Ok(mut sessions) => sessions.remove(token.as_str()),
        Err(e) => {
            error!("{:?}", e);
            return AuthrError::NotAuthorized.into_response();
        }
    };

    let pkce_verifier = match pkce_verifier {
        Some(verifier) => verifier,
        None => {
            return AuthrError::NotAuthorized.into_response();
        }
    };

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_token`.
    let user_info =
        match get_google_user_info(pkce_verifier, code, state.google_client.client.clone()).await {
            Ok(u) => u,
            Err(_) => {
                return AuthrError::NotAuthorized.into_response();
            }
        };

    let retrieved = match retrieve_or_create_user(user_info, state.clone()).await {
        Some(r) => r,
        None => {
            return AuthrError::NotAuthorized.into_response();
        }
    };

    debug!("{:?}", retrieved);

    // Generate a PKCE challenge for a new session_id & set cookie
    let (_pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let cookie_exp_duration = time::Duration::minutes(10);
    match state.sessions.lock() {
        Ok(mut sessions) => {
            let now = time::OffsetDateTime::now_utc();
            let expires = now.checked_add(cookie_exp_duration);
            match expires {
                Some(expires) => {
                    sessions.insert(pkce_verifier.secret().clone(), (retrieved, expires));
                }
                None => {
                    error!("Could not add {:?} and {:?}", now, cookie_exp_duration);
                    return AuthrError::NotAuthorized.into_response();
                }
            }
        }
        Err(e) => {
            error!("{:?}", e);
            return AuthrError::NotAuthorized.into_response();
        }
    };

    let pkce_str = pkce_verifier.into_secret();
    let cookie = Cookie::build(("session_id", pkce_str.as_str()))
        .path("/")
        .max_age(cookie_exp_duration)
        .http_only(true)
        .build();

    (
        StatusCode::TEMPORARY_REDIRECT,
        AppendHeaders([(SET_COOKIE, cookie.to_string().as_str()), (LOCATION, "/")]),
    )
        .into_response()
}

async fn get_google_user_info(
    pkce_verifier: String,
    code: String,
    client: SetClient,
) -> Result<GoogleUserInfo, ()> {
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Now you can trade it for an access token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(&http_client)
        .await
        .unwrap();

    let oauth_google_url_api = "https://www.googleapis.com/oauth2/v2/userinfo";

    let user_data = http_client
        .get(oauth_google_url_api)
        .query(&[("access_token", token_result.access_token().secret())])
        .send()
        .await;
    let user_data = match user_data {
        Ok(user_data) => user_data.text().await,
        Err(e) => {
            error!("{:?}", e);
            return Err(());
        }
    };
    match user_data {
        Ok(user_data) => {
            let user_info: std::result::Result<GoogleUserInfo, _> =
                serde_json::from_str(&user_data);
            match user_info {
                Ok(user_info) => Ok(user_info),
                Err(e) => {
                    error!("{:?}", e);
                    return Err(());
                }
            }
        }
        Err(e) => {
            error!("{:?}", e);
            return Err(());
        }
    }
}

async fn retrieve_or_create_user(user_info: GoogleUserInfo, state: Arc<AuthState>) -> Option<User> {
    let user = RequestUser::from(user_info);
    let mut retrieved: Vec<User> =
        state
            .store
            .clone()
            .get_queries::<User>(vec![QueryTypes::UserQuery(UserQuery::ByGuid(
                UserByGuid::new(user.guid.clone().unwrap()),
            ))]);
    match retrieved.len() {
        1 => retrieved.pop(),
        0 => {
            info!("Creating new user {:?}", user);
            match state.store.clone().create(user) {
                Ok(user) => {
                    info!("Created {:?}", user);
                    Some(user)
                }
                Err(e) => {
                    error!("Could not create user: {:?}", e);
                    None
                }
            }
        }
        l => {
            error!("Found {} users with guid {}", l, user.guid.unwrap().clone());
            None
        }
    }
}
