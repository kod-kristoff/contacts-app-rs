use axum::{
    extract::{FromRef, Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_flash::{Flash, IncomingFlashes, Level};
use axum_template::{engine::Engine, Key, RenderHtml};
use minijinja::{path_loader, Environment};

use crate::model::{Contact, MemContactRepo, SharedContactRepo};

pub type AppEngine = Engine<Environment<'static>>;

#[derive(Clone, FromRef)]
pub struct AppState {
    engine: AppEngine,
    contact_repo: SharedContactRepo,
    flash_config: axum_flash::Config,
}

pub fn create_app() -> Router {
    let mut jinja = Environment::new();
    jinja.set_loader(path_loader("templates"));
    jinja.add_function("get_flashed_messages", get_flashed_messages);
    let repo = MemContactRepo::shared_from_path("contacts.json");
    Router::new()
        .route("/", get(|| async { Redirect::to("/contacts") }))
        .route("/contacts", get(contacts))
        .route(
            "/contacts/new",
            get(get_contacts_new).post(post_contacts_new),
        )
        .route("/contacts/:contact_id/delete", post(contacts_delete))
        .route(
            "/contacts/:contact_id/edit",
            get(contacts_edit_get).post(contacts_edit_post),
        )
        .route("/contacts/:contact_id", get(contact_view))
        .with_state(AppState {
            engine: Engine::from(jinja),
            contact_repo: repo,
            flash_config: axum_flash::Config::new(axum_flash::Key::generate()),
        })
}

fn get_flashed_messages(
    state: &minijinja::State,
    _args: Vec<minijinja::Value>,
) -> Result<minijinja::Value, minijinja::Error> {
    match state.lookup("messages") {
        None => Ok(minijinja::Value::from(())),
        Some(messages) => Ok(minijinja::Value::from_iter(
            messages
                .try_iter()?
                .map(|v| v.get_item_by_index(1).unwrap().clone()),
        )),
    }
}
// Our state type must implement this trait. That is how the config
// is passed to axum-flash in a type safe way.
// impl FromRef<AppState> for axum_flash::Config {
//     fn from_ref(state: &AppState) -> axum_flash::Config {
//         state.flash_config.clone()
//     }
// }

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexState {
    q: Option<String>,
    contacts: Vec<Contact>,
    messages: Vec<(Level, String)>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContactsParams {
    q: Option<String>,
}

async fn contacts(
    engine: AppEngine,
    State(state): State<AppState>,
    Query(params): Query<ContactsParams>,
    flashes: IncomingFlashes,
) -> impl IntoResponse {
    let mut messages = Vec::new();
    for (level, text) in &flashes {
        messages.push((level, text.to_string()));
    }
    dbg!(&params);
    let contacts = match &params.q {
        None => state.contact_repo.all().await,
        Some(search) => state.contact_repo.search(search).await,
    };
    let state = IndexState {
        q: params.q,
        contacts,
        messages,
    };
    dbg!(&state);
    (
        flashes,
        RenderHtml(Key("index.html".to_owned()), engine, state),
    )
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NewContactCtx {
    contact: Contact,
}

async fn get_contacts_new(engine: AppEngine) -> impl IntoResponse {
    RenderHtml(
        Key("new.html".to_owned()),
        engine,
        NewContactCtx {
            contact: Contact::default(),
        },
    )
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewContact {
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
    email: Option<String>,
}

impl From<NewContact> for Contact {
    fn from(value: NewContact) -> Self {
        Self::new(value.first_name, value.last_name, value.phone, value.email)
    }
}
async fn post_contacts_new(
    engine: AppEngine,
    State(state): State<AppState>,
    flash: Flash,
    Form(new_contact): Form<NewContact>,
) -> Response {
    let contact = Contact::from(new_contact);
    match state.contact_repo.save(contact).await {
        Ok(()) => (
            flash.info("Created new contact!"),
            Redirect::to("/contacts"),
        )
            .into_response(),
        Err(contact) => RenderHtml(
            Key("new.html".to_owned()),
            engine,
            NewContactCtx { contact },
        )
        .into_response(),
    }
}

async fn contact_view(
    engine: AppEngine,
    State(state): State<AppState>,
    Path(contact_id): Path<u64>,
) -> impl IntoResponse {
    let contact = state
        .contact_repo
        .find(contact_id)
        .await
        .expect("a existing id");
    RenderHtml(
        Key("show.html".to_owned()),
        engine,
        NewContactCtx { contact },
    )
}

async fn contacts_edit_get(
    engine: AppEngine,
    State(state): State<AppState>,
    Path(contact_id): Path<u64>,
) -> impl IntoResponse {
    let contact = state
        .contact_repo
        .find(contact_id)
        .await
        .expect("a existing id");
    RenderHtml(
        Key("edit.html".to_owned()),
        engine,
        NewContactCtx { contact },
    )
}

async fn contacts_edit_post(
    engine: AppEngine,
    State(state): State<AppState>,
    flash: Flash,
    Path(contact_id): Path<u64>,
    Form(new_contact): Form<NewContact>,
) -> Response {
    let mut contact = state.contact_repo.find(contact_id).await.unwrap();
    let NewContact {
        first_name,
        last_name,
        phone,
        email,
    } = new_contact;
    contact.update(first_name, last_name, phone, email);

    match state.contact_repo.save(contact).await {
        Ok(()) => (flash.info("Updated contact!"), Redirect::to("/contacts")).into_response(),
        Err(contact) => RenderHtml(
            Key("edit.html".to_owned()),
            engine,
            NewContactCtx { contact },
        )
        .into_response(),
    }
}

async fn contacts_delete(
    State(state): State<AppState>,
    flash: Flash,
    Path(contact_id): Path<u64>,
) -> impl IntoResponse {
    let contact = state.contact_repo.find(contact_id).await.unwrap();

    state.contact_repo.delete(contact).await;
    (flash.info("Deleted contact!"), Redirect::to("/contacts"))
}
