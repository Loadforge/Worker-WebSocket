use actix_web::{web, App, HttpServer};
mod models; 
mod ws; 

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server started em ws://127.0.0.1:8080/ws");

    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(ws::ws_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
