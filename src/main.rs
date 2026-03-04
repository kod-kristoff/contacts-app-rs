mod app;
mod model;

use app::create_app;
use tokio::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let fmt_layer = fmt::layer();
    let subscriber = Registry::default().with(env_filter).with(fmt_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
    let app = create_app();

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("valid address");
    tracing::info!("Listening at {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}
