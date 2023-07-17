extern crate actix_web;

use actix_web::{middleware, App, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct InnerItem {
    pub inner_data: u64,
}
#[derive(Debug, Deserialize, Serialize)]
struct Item {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub data: String,
    pub list_of_items: Vec<InnerItem>,
}

impl Item {
    pub fn new(data: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            data,
            list_of_items: vec![],
        }
    }
}

impl InnerItem {
    pub fn new(inner_data: u64) -> Self {
        Self { inner_data }
    }
}
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let a = Item::new(String::from("Data"));
    println!("{:?}", a);

    HttpServer::new(|| App::new().wrap(middleware::Logger::default()))
        .bind("0.0.0.0:9090")?
        .run()
        .await
}
