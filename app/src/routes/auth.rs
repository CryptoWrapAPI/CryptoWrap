use crate::AUTH_TAG;
use crate::AppState;
use crate::entity::tokens;
use axum::{Json, extract::State, http::StatusCode, routing::post};
use base64::Engine;
use openssl::rand::rand_bytes;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
    // pub token_type: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AuthRequest {
    pub token: String,
}

/// Generate a secure bearer token
///
/// Returns a newly generated cryptographically secure random bearer token.
#[utoipa::path(
    get,
    path = "/token", // API key (permament, can be refreshed/rotated), not JWT (no db lookup, temporary, can expire, contain data encrypted)
    tag = AUTH_TAG,
    responses(
        (status = 200, description = "Token generated successfully", body = TokenResponse)
    )
)]
pub async fn generate_token(state: State<AppState>) -> Json<TokenResponse> {
    let token_prefix = &state.token_prefix;

    let mut bytes = [0u8; 90];
    rand_bytes(&mut bytes).expect("Failed to generate secure random bytes");

    // Encode as hex for the token
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let token = format!("{}{}", token_prefix, token);

    Json(TokenResponse {
        token, //token prefixed_
               // token_type: "Bearer".to_string(),
    })
}

// no need to include this endpoint in openapi specification, because it's used only for web app (dashboard) to acquire session cookie
// #[utoipa::path(
//     post,
//     path = "/login_or_register",
//     request_body = AuthRequest,
//     responses(
//         (status = 200, description = "Authentication successful"),
//         (status = 400, description = "Invalid token")
//     ),
// )]
async fn login_or_register(
    state: State<AppState>,
    cookies: Cookies,
    Json(auth_request): Json<AuthRequest>,
) -> StatusCode {
    if auth_request.token.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    if auth_request.token.len() < 100 {
        return StatusCode::BAD_REQUEST;
    }

    if auth_request.token.len() > 200 {
        return StatusCode::BAD_REQUEST;
    }

    // Strip the token prefix before hashing
    let token_without_prefix = auth_request
        .token
        .strip_prefix(&state.token_prefix)
        .unwrap_or(&auth_request.token);

    let pepper_bytes = state.blake3_hash_token_pepper.as_bytes();
    let mut key = [0u8; 32];
    let copy_len = pepper_bytes.len().min(32);
    key[..copy_len].copy_from_slice(&pepper_bytes[..copy_len]);
    let token_hash = blake3::keyed_hash(&key, token_without_prefix.as_bytes());
    let token_hash_hex = token_hash.to_hex().to_string();

    let token_model = match tokens::Entity::find()
        .filter(tokens::Column::TokenHash.eq(&token_hash_hex))
        .one(&state.conn)
        .await
        .unwrap()
    {
        Some(token) => token,
        None => {
            let new_token = tokens::ActiveModel {
                token_hash: Set(token_hash_hex),
                ..Default::default()
            };
            new_token.insert(&state.conn).await.unwrap()
            // add rate limiting: 10 tokens per hour max for 1 ip (registered, not generated or logged in)
            // how to efficiently apply rate-limiting without load on db? redis!
        }
    };

    cookies
        .private(&state.cookie_key)
        .add(Cookie::new("user_id", token_model.id.to_string()));

    StatusCode::OK
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(generate_token))
        .route("/login_or_register", post(login_or_register))
}
