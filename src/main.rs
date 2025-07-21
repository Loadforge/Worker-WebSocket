use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
mod models; 
mod ws; 

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!("Server started in ws://127.0.0.1:8080/ws");

    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(ws::ws_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
