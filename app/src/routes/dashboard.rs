// use crate::{COOKIE_NAME, KEY};
use askama::Template;
use askama_web::WebTemplate;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Router, routing::get};
// use tower_cookies::Cookies;
use crate::AppState;
use axum::extract::State;

#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {}

// async fn dashboard(cookies: Cookies) -> Response {
async fn dashboard(state: State<AppState>) -> Response {
    // async fn dashboard() -> Response {
    // let key = KEY.get().unwrap(); // can also store key in appstate
    // let private_cookies = cookies.private(key);

    // let cookie_user_id = private_cookies
    //     .get(COOKIE_NAME)
    //     .and_then(|c| c.value().parse().ok())
    //     .unwrap_or(0);

    // if cookie_user_id == 0 {
    //     // required auth cookie doesn't exist, redirect user to /auth
    //     return Redirect::to("/auth").into_response();
    // }

    // query user_id in database to identify user
    // clear cookie if user entry doesn't exist

    DashboardTemplate {}.into_response()
}

#[derive(Template, WebTemplate)]
#[template(path = "welcome.html")]
struct WelcomeTemplate {}

async fn welcome() -> WelcomeTemplate {
    WelcomeTemplate {}
}

#[derive(Template, WebTemplate)]
#[template(path = "landing.html")]
struct LandingTemplate {}

async fn landing() -> LandingTemplate {
    LandingTemplate {}
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(landing))
        .route("/auth", get(welcome))
        .route("/dashboard", get(dashboard))
        .with_state(state)
}

// check cookie with encrypted bearer token here
// if exists - check user - if valid - let to dashboard
// - if not - back to /, with optional clearing cookie
//
// add token encryption in auth.html for new logins
