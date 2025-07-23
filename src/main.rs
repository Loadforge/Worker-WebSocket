use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
mod models; 
mod ws; 
mod utils;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!("Server started in ws://localhost:8080/ws");

    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(ws::ws_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
