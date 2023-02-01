use std::env;
use actix_web as web;
use sea_orm as orm;

mod posts;
mod actix_seaorm_api;

// app

#[web::main]
async fn main() -> std::io::Result<()> {

    let db_url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => panic!("DATABASE_URL is not set !")
    };

    let conn = orm::Database::connect(&db_url).await.unwrap();
    
    println!("Starting server at: http://localhost:8080");
    web::HttpServer::new(move || {
        web::App::new()
            .app_data(web::web::Data::new(conn.clone()))
            .service(web::web::scope("/posts").configure(
                actix_seaorm_api::ModelApi::<posts::Model, posts::ActiveModel>::services)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}