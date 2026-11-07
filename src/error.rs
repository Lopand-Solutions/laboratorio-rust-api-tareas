//! Los errores de la API y cómo salen como respuesta HTTP.
use axum::{
    Json,
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NoEncontrada(i64),
    Invalida(String),
    Cuerpo(JsonRejection),
    Consulta(QueryRejection),
    BaseDeDatos(sqlx::Error),
}

impl From<JsonRejection> for ApiError {
    fn from(r: JsonRejection) -> Self {
        ApiError::Cuerpo(r)
    }
}

impl From<QueryRejection> for ApiError {
    fn from(r: QueryRejection) -> Self {
        ApiError::Consulta(r)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::BaseDeDatos(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (codigo, mensaje) = match self {
            ApiError::NoEncontrada(id) => {
                (StatusCode::NOT_FOUND, format!("no existe la tarea {id}"))
            }
            ApiError::Invalida(m) => (StatusCode::UNPROCESSABLE_ENTITY, m),
            ApiError::Cuerpo(r) => (r.status(), r.body_text()),
            ApiError::Consulta(r) => (r.status(), r.body_text()),
            ApiError::BaseDeDatos(e) => {
                // El detalle va al log, no al cliente.
                eprintln!("error de base de datos: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "error interno".to_string(),
                )
            }
        };
        (codigo, Json(json!({ "error": mensaje }))).into_response()
    }
}
