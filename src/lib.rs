//! API de tareas: el router vive aquí para que las pruebas lo usen sin abrir un puerto.
pub mod error;
pub mod tareas;

use axum::{Router, routing::get};
use sqlx::PgPool;

pub fn app(db: PgPool) -> Router {
    Router::new()
        .route("/salud", get(|| async { "ok" }))
        .route("/tareas", get(tareas::listar).post(tareas::crear))
        .route(
            "/tareas/{id}",
            get(tareas::obtener)
                .patch(tareas::actualizar)
                .delete(tareas::borrar),
        )
        .with_state(db)
}
