use serde::{Serialize, Deserialize};
use actix_web as web;
use sea_orm as orm;
use sea_orm::{EntityTrait, Iterable, PrimaryKeyToColumn, QuerySelect};


pub struct ModelApi<Model, ActiveModel>
{
    phantom1: std::marker::PhantomData<Model>,
    phantom2: std::marker::PhantomData<ActiveModel>,
}

impl<Model: orm::ModelTrait + 'static, ActiveModel: orm::ActiveModelTrait<Entity = <Model as orm::ModelTrait>::Entity> + orm::ActiveModelBehavior + std::marker::Send + 'static> ModelApi<Model, ActiveModel>
where
    Model : Serialize + for<'a> Deserialize<'a>,
    <<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::Model : Serialize + for<'a> Deserialize<'a>,
    <<<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::PrimaryKey as orm::PrimaryKeyTrait>::ValueType : Serialize + for<'a> Deserialize<'a>,
    Model : orm::IntoActiveModel<ActiveModel>,
{

    pub fn services(cfg: &mut web::web::ServiceConfig) {
        cfg.service(
            web::web::resource("/")
                .route(web::web::get().to(Self::list_model))
                .route(web::web::post().to(Self::create_model))
        );
        cfg.service(
            web::web::resource("/{item_id}")
                .route(web::web::get().to(Self::get_model))
                .route(web::web::delete().to(Self::delete_model))
        );
    }

    pub async fn get_model(
        conn: web::web::Data<orm::DatabaseConnection>,
        item_id: web::web::Path<<<<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::PrimaryKey as orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<web::web::Json<<<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::Model>, web::Error> {
    
        let item_id_val = item_id.into_inner();
        match <Model as orm::ModelTrait>::Entity::find_by_id(item_id_val).one(conn.get_ref()).await {
            Ok(Some(item)) => Ok(web::web::Json(item)),
            _ => Err(web::error::ErrorNotFound("Not Found"))
        }
    }

    async fn create_model(
        conn: web::web::Data<orm::DatabaseConnection>,
        form_data: web::web::Form<Model>,
    ) -> Result<web::web::Json<ResId<Model>>, web::Error> {

        let mut new_item = form_data.into_inner().into_active_model();
        unset_primary_key(&mut new_item);
        match <ActiveModel as orm::ActiveModelTrait>::Entity::insert(new_item).exec(conn.get_ref()).await {
            Ok(insert_result) => {
                Ok(web::web::Json(ResId::<Model>{
                    id: insert_result.last_insert_id
                }))
            },
            Err(_) => Err(web::error::ErrorInternalServerError("Internal Error")),
        }
    }

    pub async fn delete_model(
        conn: web::web::Data<orm::DatabaseConnection>,
        item_id: web::web::Path<<<<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::PrimaryKey as orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<web::HttpResponse, web::Error> {
        
        let item_id_val = item_id.into_inner();
        match <Model as orm::ModelTrait>::Entity::delete_by_id(item_id_val).exec(conn.get_ref()).await {
            Err(_) => Err(web::error::ErrorNotFound("Not Found")),
            Ok(res) => match res.rows_affected {
                0 => return Err(web::error::ErrorNotFound("Not Found")),
                _ => Ok(web::HttpResponse::Ok().body(""))
            }
        }
    }

    async fn list_model(
        conn: web::web::Data<orm::DatabaseConnection>,
    ) -> Result<web::web::Json<ResList<<<Model as sea_orm::ModelTrait>::Entity as sea_orm::EntityTrait>::Model>>, web::Error>{
        match <Model as orm::ModelTrait>::Entity::find().limit(10).all(conn.get_ref()).await {
            Err(_) => Err(web::error::ErrorInternalServerError("Internal Error")),
            Ok(items) => Ok(web::web::Json(ResList{
                items: items
            })),
        }
    }
}

#[derive(Serialize)]
struct ResId<Model: orm::ModelTrait>
where
    <<<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::PrimaryKey as orm::PrimaryKeyTrait>::ValueType : Serialize,
{
    pub id: <<<Model as orm::ModelTrait>::Entity as orm::EntityTrait>::PrimaryKey as orm::PrimaryKeyTrait>::ValueType,
}


#[derive(Serialize)]
struct ResList<Model: orm::ModelTrait>
where
    Model : Serialize,
{
    pub items: Vec<Model>,
}

fn unset_primary_key<ActiveModel: orm::ActiveModelTrait>(model: &mut ActiveModel) {
    let mut cols = <<ActiveModel as orm::ActiveModelTrait>::Entity as orm::EntityTrait>::PrimaryKey::iter();
    while let Some(col) = cols.next() {
        model.not_set(col.into_column());
    }
}