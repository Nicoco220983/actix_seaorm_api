use std::env;
use std::marker::PhantomData;
use serde::{Serialize, Deserialize};
use actix_web::{get, post, web, error, App, HttpResponse, HttpServer, HttpRequest, Responder, Error};
use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait, ModelTrait, PrimaryKeyTrait, ActiveModelTrait, ActiveModelBehavior, IntoActiveModel, Set, Iterable};
use async_trait::async_trait;
use derive_more::{Display, Error};
use sea_orm::PrimaryKeyToColumn;


pub struct ModelApi<Model, ActiveModel>
{
    phantom1: PhantomData<Model>,
    phantom2: PhantomData<ActiveModel>,
}

impl<Model: ModelTrait + 'static, ActiveModel: ActiveModelTrait<Entity = <Model as ModelTrait>::Entity> + ActiveModelBehavior + std::marker::Send + 'static> ModelApi<Model, ActiveModel>
where
    Model : Serialize + for<'a> Deserialize<'a>,
    <<Model as ModelTrait>::Entity as EntityTrait>::Model : Serialize + for<'a> Deserialize<'a>,
    <<<Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType : Serialize + for<'a> Deserialize<'a>,
    Model : IntoActiveModel<ActiveModel>,
{

    pub fn services(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::resource("/")
                .route(web::post().to(Self::create_model))
        );
        cfg.service(
            web::resource("/{item_id}")
                .route(web::get().to(Self::get_model))
                .route(web::delete().to(Self::delete_model))
        );
    }

    pub async fn get_model(
        conn: web::Data<DatabaseConnection>,
        item_id: web::Path<<<<Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    ) -> Result<web::Json<<<Model as ModelTrait>::Entity as EntityTrait>::Model>, Error> {
    
        let item_id_val = item_id.into_inner();
        let item: <<Model as ModelTrait>::Entity as EntityTrait>::Model = <Model as ModelTrait>::Entity::find_by_id(item_id_val).one(conn.get_ref())
            .await
            .expect("could not find item")
            .unwrap_or_else(|| panic!("could not find item"));
        
        Ok(web::Json(item))
    }

    async fn create_model(
        conn: web::Data<DatabaseConnection>,
        form_data: web::Form<Model>,
    ) -> Result<web::Json<ResId<Model>>, OrmError> {

        let mut new_item = form_data.into_inner().into_active_model();
        unset_primary_key(&mut new_item);
        match <ActiveModel as ActiveModelTrait>::Entity::insert(new_item).exec(conn.get_ref()).await {
            Ok(insert_result) => {
                Ok(web::Json(ResId::<Model>{
                    id: insert_result.last_insert_id
                }))
            },
            Err(e) => Err(OrmError::InternalError),
        }
    }

    pub async fn delete_model(
        conn: web::Data<DatabaseConnection>,
        item_id: web::Path<<<<Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    ) -> Result<HttpResponse, Error> {
        
        let item_id_val = item_id.into_inner();
        match <Model as ModelTrait>::Entity::delete_by_id(item_id_val).exec(conn.get_ref()).await {
            Err(_) => Err(error::ErrorNotFound("Not found")),
            Ok(res) => match res.rows_affected {
                0 => return Err(error::ErrorNotFound("Not found")),
                _ => Ok(HttpResponse::Ok().body(""))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ResId<Model: ModelTrait>
where
    <<<Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType : Serialize,
    <<<Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType : for<'a> Deserialize<'a>,
{
    pub id: <<<Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
}

#[derive(Debug, Display, Error)]
enum OrmError {
    #[display(fmt = "internal error")]
    InternalError,
}

impl error::ResponseError for OrmError {}


fn unset_primary_key<ActiveModel: ActiveModelTrait>(model: &mut ActiveModel) {
    let mut cols = <<ActiveModel as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey::iter();
    while let Some(col) = cols.next() {
        model.not_set(col.into_column());
    }
}