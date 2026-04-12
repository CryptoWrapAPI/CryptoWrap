use crate::AppState;
use crate::entity::tokens;
use axum::http::HeaderMap;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
// use uuid::Uuid;

/// Extract and validate API key from headers, return token ID if valid
// pub async fn extract_api_key(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
pub async fn extract_user_row(state: &AppState, headers: &HeaderMap) -> Option<tokens::Model> {
    let api_key = headers.get("X-API-Key").and_then(|v| v.to_str().ok())?;

    let token_without_prefix = api_key.strip_prefix(&state.token_prefix).unwrap_or(api_key);

    let pepper_bytes = state.blake3_hash_token_pepper.as_bytes();
    let mut key = [0u8; 32];
    let copy_len = pepper_bytes.len().min(32);
    key[..copy_len].copy_from_slice(&pepper_bytes[..copy_len]);
    let token_hash = blake3::keyed_hash(&key, token_without_prefix.as_bytes());
    let token_hash_hex = token_hash.to_hex().to_string();

    let token_model = tokens::Entity::find()
        .filter(tokens::Column::TokenHash.eq(&token_hash_hex))
        .one(&state.conn)
        .await
        .ok()??;

    Some(token_model)
}
