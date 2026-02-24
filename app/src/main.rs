mod routes;
use routes::auth;
use routes::dashboard;
use routes::payment;
use sea_orm::{Database, DatabaseConnection};
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tower_cookies::{CookieManagerLayer, Key};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
// use utoipa::{
//     Modify, OpenApi,
//     openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
// };
use dotenvy::dotenv;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
mod entity;

use hex;
use std::env;
// use migration::{Migrator, MigratorTrait};

const AUTH_TAG: &str = "Authentication";
const PAYMENT_TAG: &'static str = "Payments";

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token_prefix = env::var("TOKEN_PREFIX").expect("TOKEN_PREFIX must be set");

    let db_url = env::var("DB_URL").expect("DB_URL must be set");
    let app_key = env::var("APP_KEY").expect("APP_KEY must be set");
    let blake3_hash_token_pepper =
        env::var("BLAKE3_HASH_TOKEN_PEPPER").expect("BLAKE3_HASH_TOKEN_PEPPER must be set");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    // Migrator::up(&conn, None).await.unwrap();

    let state = AppState {
        conn,
        token_prefix,
        blake3_hash_token_pepper,
        cookie_key: Key::from(&hex::decode(app_key).unwrap()),
    };

    #[derive(OpenApi)]
    #[openapi(
        // modifiers(&SecurityAddon),
        info(
            title = "CryptoWrap",
            version = "0.1.3",
            // description = "test",
            // license = "https://codeberg.org/NakataModem/cryptowrap/raw/branch/main/LICENSE.md", // license struct
            // terms_of_service = "/assets/tos.html", // TODO
            // contact = , // contact struct
            //extensions = ,
        ),
        tags(
         (name = AUTH_TAG),
         (name = PAYMENT_TAG),
        ),
    )]
    struct ApiDoc;

    // struct SecurityAddon;

    // impl Modify for SecurityAddon {
    // fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    // if let Some(components) = openapi.components.as_mut() {
    // components.add_security_scheme(
    // "api_key",
    // SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
    // )
    // }
    // }
    // }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();

    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1/auth", auth::router())
        .nest("/api/v1/payments", payment::router())
        .layer(CookieManagerLayer::new())
        .with_state(state)
        .split_for_parts();

    let static_files = ServeDir::new("./assets");

    let router = dashboard::router()
        .merge(api_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .nest_service("/assets", static_files)
        .layer(TraceLayer::new_for_http());

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;

    tracing::info!("Serving!");

    axum::serve(listener, router.into_make_service()).await
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
    token_prefix: String,
    blake3_hash_token_pepper: String,
    cookie_key: Key,
}
