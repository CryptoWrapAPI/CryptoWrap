use askama::Template;
use askama_web::WebTemplate;
use axum::{Router, routing::get};

#[derive(Template, WebTemplate)]
#[template(path = "iframe_v1.html")]
struct IframeV1Template;

async fn iframe_v1() -> IframeV1Template {
    IframeV1Template
}

pub fn router() -> Router {
    Router::new().route("/iframe/v1", get(iframe_v1))
}
