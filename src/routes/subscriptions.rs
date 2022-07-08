use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn subscribe(form: web::Form<FormData>, poll: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query!(
        r#"
        insert into subscriptions(id, email, name, subscribed_at) 
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(poll.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Faled to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
