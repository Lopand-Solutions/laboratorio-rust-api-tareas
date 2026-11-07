use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").expect("falta DATABASE_URL");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&url)
        .await
        .expect("no pude conectar a Postgres");
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("falló una migración");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("escuchando en http://{}", listener.local_addr().unwrap());
    axum::serve(listener, api_tareas::app(db))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
        .unwrap();
}
