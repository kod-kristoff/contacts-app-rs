use axum::{
    extract::{FromRef, Query},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_template::{engine::Engine, Key, RenderHtml};
use minijinja::{path_loader, Environment};

use crate::model::Contact;

pub type AppEngine = Engine<Environment<'static>>;

#[derive(Clone, FromRef)]
pub struct AppState {
    engine: AppEngine,
}

pub fn create_app() -> Router {
    let mut jinja = Environment::new();
    jinja.set_loader(path_loader("templates"));
    Router::new()
        .route("/", get(|| async { Redirect::to("/contacts") }))
        .route("/contacts", get(contacts))
        .with_state(AppState {
            engine: Engine::from(jinja),
        })
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexState {
    q: Option<String>,
    contacts: Vec<Contact>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContactsParams {
    q: Option<String>,
}

async fn contacts(engine: AppEngine, Query(params): Query<ContactsParams>) -> impl IntoResponse {
    dbg!(&params);
    // let q = match q {
    //     None => None,
    //     Some(Query(search)) => Some(search),
    // };
    let state = IndexState {
        q: params.q,
        contacts: Vec::new(),
    };
    dbg!(&state);
    RenderHtml(Key("index.html".to_owned()), engine, state)
}
