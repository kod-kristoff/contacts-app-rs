mod app;
mod model;

use app::create_app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = create_app();

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("valid address");
    println!("Listening at {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}
