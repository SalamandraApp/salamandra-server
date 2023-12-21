use crate::models::users::User;
use crate::schema::users::dsl::*;
use diesel::prelude::*;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/register", web::post().to(login));
}

async fn login(login_info: web::Json<LoginRequest>) -> Result<HttpResponse> {
   
    let result = web::block(move || {
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        let count = users.filter(username.eq(&login_info.username))
            .count()
            .get_result::<i64>(conn);
        count
    })
    .await;

    match result {
        Ok(Ok(res_count)) => {
            if res_count == 0 {
                println!("user NOT in the database.");
            } else {
                println!("user is in the database.");
            }
        },
        Ok(Err(_)) => todo!(),
        Err(_) => todo!()
    }

    Ok(HttpResponse::Ok().body("Processed login request"))
}
