use axum::{response::Redirect, routing::get, Router};

pub fn create_app() -> Router {
    Router::new().route("/", get(|| async { Redirect::to("/contacts") }))
}
