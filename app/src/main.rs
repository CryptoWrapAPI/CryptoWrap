mod routes;
use routes::auth;
use routes::checkout;
use routes::dashboard;
// use routes::payment;
use routes::deposit;
use routes::qr;
use sea_orm::{Database, DatabaseConnection};
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::OnceLock;
use tokio::net::TcpListener;
// use tower_cookies::{Cookie, CookieManagerLayer, Cookies, Key};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
// use utoipa::{
//     Modify, OpenApi,
//     openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
// };
use dotenvy::dotenv;
use utoipa::{
    Modify, OpenApi, openapi::security::ApiKey, openapi::security::ApiKeyValue,
    openapi::security::SecurityScheme,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
mod entity;
mod wallet;

use tg_notify::Notifier;

use hex;
use std::env;
// use migration::{Migrator, MigratorTrait};

use hex::FromHex;

const AUTH_TAG: &str = "Authentication";
const PAYMENT_TAG: &'static str = "Payments";

// const COOKIE_NAME: &str = "auth_cookie";
// static KEY: OnceLock<Key> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token_prefix = env::var("TOKEN_PREFIX").expect("TOKEN_PREFIX must be set");

    let db_url = env::var("DB_URL").expect("DB_URL must be set");
    let app_key = env::var("APP_KEY").expect("APP_KEY must be set");
    let blake3_hash_token_pepper =
        env::var("BLAKE3_HASH_TOKEN_PEPPER").expect("BLAKE3_HASH_TOKEN_PEPPER must be set");
    let monero_wallet_rpc_address =
        env::var("MONERO_WALLET_RPC_ADDRESS").expect("MONERO_WALLET_RPC_ADDRESS must be set");
    let ltc_api_url = env::var("LTC_API_URL").expect("LTC_API_URL must be set");
    let ltc_mpk = env::var("LTC_MPK").expect("LTC_MPK must be set");
    let current_url =
        env::var("CURRENT_URL").expect("CURRENT_URL must be set to construct checkout gateway");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    // Migrator::up(&conn, None).await.unwrap();

    let tg_bot_token = env::var("TG_BOT_TOKEN").expect("TG_BOT_TOKEN must be set");
    let tg_chat_id = env::var("TG_CHAT_ID").expect("TG_CHAT_ID must be set");

    let tg_notificator = Notifier::new(tg_bot_token, tg_chat_id);

    // let hex_string = env::var("COOKIE_KEY").expect("COOKIE_KEY must be set");

    // let cookie_key: &[u8] = &[0; 64]; // replace with real CSRNG key
    // let cookie_key: &[u8] = &Vec::from_hex(hex_string).expect("Invlalid hex string.");
    // let cookie_key: &[u8] = &Vec::from_hex(&app_key).expect("Invlalid hex string.");
    // KEY.set(Key::from(cookie_key)).ok(); // use external key instead of appstate because state layer is not defined for this router

    let state = AppState {
        conn,
        token_prefix,
        blake3_hash_token_pepper,
        // cookie_key: Key::from(&hex::decode(app_key).unwrap()),
        monero_wallet: wallet::monero::MoneroWallet::new(&monero_wallet_rpc_address),
        litecoin_wallet: wallet::litecoin::LitecoinWallet::new(&ltc_api_url, &ltc_mpk),
        current_url,
        tg_notificator,
    };

    #[derive(OpenApi)]
    #[openapi(
        modifiers(&SecurityAddon),
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
        components(
            schemas(
                // payment::CreateInvoiceResponse,
                // payment::Currency,
                // payment::CreateInvoiceRequest,
                // payment::CheckInvoiceRequest,
                // payment::PaymentStatus,
                // payment::CheckInvoiceResponse,
            )
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
                )
            }
        }
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();

    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1/auth", auth::router())
        // .nest("/api/v1/payments", payment::router())
        .nest("/api/v1/deposit", deposit::router()) // router is named plural, database table is also plural, but the endpoint is singular
        // ^ it implies on simplicity of deposits logic, just create and check. funds are spendable after confirmed.
        // ... nevermind, router name is singlular as well / too
        // .layer(CookieManagerLayer::new())
        .with_state(state)
        .split_for_parts();

    let static_files = ServeDir::new("./assets");

    let router = dashboard::router()
        // .layer(CookieManagerLayer::new()) // apply cookie only for dashbourd router (endpoints: /dashboard and /auth)
        .merge(checkout::router())
        .merge(qr::router())
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
    // cookie_key: Key,
    monero_wallet: wallet::monero::MoneroWallet,
    litecoin_wallet: wallet::litecoin::LitecoinWallet,
    current_url: String,
    tg_notificator: Notifier, // add easy switch to enable/disable notification
}

// add some sort of panic/error handler (e.g. custom) to send notification about it (e.g. alert system in telegram bot)
