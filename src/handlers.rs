use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use crate::db::CouchDB;
use serde_json::Value;


pub async fn get_document(db: web::Data<Arc<CouchDB>>, id: web::Path<String>) -> impl Responder {
    match db.get_document(&id).await {
        Ok(doc) => HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::NotFound().body(format!("Document with id {} not found", id))
        }
    }
}

pub async fn put_document(db: web::Data<Arc<CouchDB>>, id: web::Path<String>, data: web::Json<Value>) -> impl Responder {
    match db.put_document(&id, data.into_inner()).await {
        Ok(doc) => HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}
