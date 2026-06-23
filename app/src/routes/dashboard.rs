// use crate::{COOKIE_NAME, KEY};
use askama::Template;
use askama_web::WebTemplate;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{
    Router,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use hyper::StatusCode;
use uuid::Uuid;
// use tower_cookies::Cookies;
use crate::AppState;
use crate::entity::prelude::*;
use crate::entity::deposits;
use axum::extract::State;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    user_uuid: String,
    member_since: String,
    deposit_requests_count: String,
    successful_deposits_count: String,
}

// async fn dashboard(cookies: Cookies) -> Response {
async fn dashboard(state: State<AppState>, jar: PrivateCookieJar) -> (PrivateCookieJar, Response) {
    // async fn dashboard() -> Response {
    // let key = KEY.get().unwrap(); // can also store key in appstate
    // let private_cookies = cookies.private(key);

    // let cookie_user_id = private_cookies
    //     .get(COOKIE_NAME)
    //     .and_then(|c| c.value().parse().ok())
    //     .unwrap_or(0);

    // if cookie_user_id == 0 {
    // required auth cookie doesn't exist, redirect user to /auth
    // return Redirect::to("/auth").into_response();
    // }

    let user_token_entry;

    if let Some(user_id) = jar.get("auth") {
        // verify user id existance in db
        // println!("user_id: {}", user_id.value());
        let token_id_str = user_id.value();

        match token_id_str.parse::<Uuid>() {
            Ok(token_id) => {
                match Tokens::find_by_id(token_id).one(&state.conn).await {
                    Ok(Some(token)) => {
                        // user identified
                        println!("Found token: {:?}", token);
                        user_token_entry = token;
                    }
                    Ok(None) => {
                        println!("Token not found: {}", token_id);
                        // clear cookie
                        let jar = jar
                            .remove(Cookie::from("auth"))
                            .remove(Cookie::from("auth_js"));
                        // return (jar, Redirect::to("/auth").into_response());
                        return (jar, Redirect::to("/").into_response());
                    }
                    Err(e) => {
                        // response with 500 error?
                        eprintln!("Database error: {}", e);
                        return (
                            jar,
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Database error: {}", e),
                            )
                                .into_response(),
                        );
                    }
                }
            }
            Err(_) => {
                // token uuid is invalid
                // clear cookie
                let jar = jar
                    .remove(Cookie::from("auth"))
                    .remove(Cookie::from("auth_js"));
                return (jar, Redirect::to("/").into_response());
            }
        }
    } else {
        let jar = jar.remove(Cookie::from("auth_js")); // remove auth_js to make sure dashboard.js won't trigger redirect
        // no auth cookie - redirect to auth page
        return (jar, Redirect::to("/").into_response());
    }

    // user database entry is available to render dashboard
    // user_token_entry

    let member_since = user_token_entry.created_at.format("%Y-%m-%d").to_string();

    let deposit_requests_count = Deposits::find()
        .filter(deposits::Column::OwnerId.eq(user_token_entry.id))
        .count(&state.conn)
        .await
        .unwrap_or(0)
        .to_string();
    let successful_deposits_count = Deposits::find()
        .filter(deposits::Column::OwnerId.eq(user_token_entry.id))
        .filter(deposits::Column::PaymentStatus.eq("confirmed"))
        .count(&state.conn)
        .await
        .unwrap_or(0)
        .to_string();

    (
        jar,
        DashboardTemplate {
            user_uuid: user_token_entry.id.to_string(),
            member_since,
            deposit_requests_count,
            successful_deposits_count,
        }
        .into_response(),
    )
}

// #[derive(Template, WebTemplate)]
// #[template(path = "welcome.html")]
// struct WelcomeTemplate {}

// async fn welcome(jar: PrivateCookieJar) -> Response {
//     // check if auth cookie present
//     if let Some(_user_id) = jar.get("auth") {
//         // redirect authenticated user to dashboard
//         return Redirect::to("/dashboard").into_response();
//     }

//     // no auth cookie - continue to authenticate

//     WelcomeTemplate {}.into_response()
// }

#[derive(Template, WebTemplate)]
#[template(path = "landing.html")]
struct LandingTemplate {}

async fn landing() -> LandingTemplate {
    LandingTemplate {}
}

async fn logout(jar: PrivateCookieJar) -> (PrivateCookieJar, StatusCode) {
    // Clear the auth cookie
    let updated_jar = jar
        .remove(Cookie::from("auth"))
        .remove(Cookie::from("auth_js"));
    (updated_jar, StatusCode::OK)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(landing))
        // .route("/auth", get(welcome))
        .route("/dashboard", get(dashboard))
        .route("/logout", post(logout))
        .with_state(state)
}

// check cookie with encrypted bearer token here
// if exists - check user - if valid - let to dashboard
// - if not - back to /, with optional clearing cookie
//
// add token encryption in auth.html for new logins

// TODO:
// add feature to clear cookies if token is invalid (uuid format) or is not found in database
