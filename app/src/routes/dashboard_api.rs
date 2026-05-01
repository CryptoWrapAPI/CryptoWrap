use crate::AppState;
use crate::entity::tokens;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{
    Router,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use hyper::StatusCode;
use sea_orm::{EntityTrait, QueryFilter};
use uuid::Uuid;

async fn thing(jar: PrivateCookieJar) -> (PrivateCookieJar, StatusCode) {
    (jar, StatusCode::OK)
}

pub fn router(state: AppState) -> Router {
    Router::new().route("/some", post(thing)).with_state(state)
}
