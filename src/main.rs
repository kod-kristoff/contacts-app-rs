mod app;
mod model;

use app::create_app;

#[tokio::main]
async fn main() {
    let app = create_app();

    let address = "127.0.0.1:3000".parse().expect("valid address");
    println!("Listening at {address}");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
