use std::env;
use serde::{Serialize, Deserialize};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, HttpRequest, Responder, Error};
use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait, ActiveModelTrait, Set};

mod posts;
mod actix_seaorm_api;

fn services_posts(cfg: &mut web::ServiceConfig) {
    // cfg.service(get_post);
    cfg.service(create_post);
}

// #[get("/{post_id}")]
// async fn get_post(
//     data: web::Data<AppState>,
//     post_id: web::Path<i32>,
// ) -> Result<web::Json<posts::Model>, Error> {

//     let conn = &data.conn;

//     let post_id_val = post_id.into_inner();
//     let post: posts::Model = posts::Entity::find_by_id(post_id_val).one(conn)
//         .await
//         .expect("could not find post")
//         .unwrap_or_else(|| panic!("could not find post with id {}", post_id_val));
    
//     Ok(web::Json(post))
// }

#[derive(Serialize, Deserialize)]
struct ResId {
    pub id: i32,
}

#[post("/")]
async fn create_post(
    data: web::Data<AppState>,
    form_data: web::Form<posts::Model>,
) -> Result<web::Json<ResId>, Error> {

    let conn = &data.conn;

    let new_post = posts::ActiveModel {
        name: Set(form_data.name.to_owned()),
        ..Default::default()
    }
    .save(conn)
    .await
    .expect("could not insert post");

    Ok(web::Json(ResId{
        id: new_post.id.unwrap()
    }))
}

// pub async fn find_post_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<posts::Model>, DbErr> {
//     posts::Entity::find_by_id(id).one(db).await
// }

// /// If ok, returns (post models, num pages).
// pub async fn find_posts_in_page(
//     db: &DbConn,
//     page: u64,
//     posts_per_page: u64,
// ) -> Result<(Vec<post::Model>, u64), DbErr> {
//     // Setup paginator
//     let paginator = Post::find()
//         .order_by_asc(post::Column::Id)
//         .paginate(db, posts_per_page);
//     let num_pages = paginator.num_pages().await?;

//     // Fetch paginated posts
//     paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
// }

// endpoints

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// app

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let db_url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => panic!("DATABASE_URL is not set !")
    };

    let conn = Database::connect(&db_url).await.unwrap();
    
    println!("Starting server at: http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conn.clone()))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            // .service(web::scope("/posts").configure(posts::Entity::services))
            .service(web::scope("/posts").configure(actix_seaorm_api::ModelApi::<posts::Model, posts::ActiveModel>::services))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}