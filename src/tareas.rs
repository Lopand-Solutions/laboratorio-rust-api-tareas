//! Tareas: validación, consultas a Postgres y handlers.
use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::ApiError;

pub const POR_PAGINA: i64 = 20;
const TITULO_MAXIMO: usize = 200;

#[derive(Debug, PartialEq, Serialize)]
pub struct Tarea {
    pub id: i64,
    pub titulo: String,
    pub hecha: bool,
    pub prioridad: i16,
}

#[derive(Deserialize)]
pub struct NuevaTarea {
    pub titulo: String,
    pub prioridad: Option<i16>,
}

#[derive(Default, Deserialize)]
pub struct Cambios {
    pub titulo: Option<String>,
    pub hecha: Option<bool>,
    pub prioridad: Option<i16>,
}

#[derive(Deserialize)]
pub struct Filtro {
    pub hecha: Option<bool>,
    pub pagina: Option<i64>,
}

// ---------- Validación: funciones puras, se prueban sin base de datos ----------

pub fn validar_titulo(titulo: &str) -> Result<String, ApiError> {
    todo!("recorta el título; vacío o de más de 200 caracteres es ApiError::Invalida")
}

pub fn validar_prioridad(prioridad: i16) -> Result<i16, ApiError> {
    todo!("la prioridad va de 1 a 3; otra es ApiError::Invalida")
}

// ---------- Consultas: una función por operación, sin nada de HTTP ----------

pub async fn insertar(db: &PgPool, titulo: &str, prioridad: i16) -> sqlx::Result<Tarea> {
    todo!("INSERT … RETURNING con query_as!")
}

pub async fn buscar(db: &PgPool, id: i64) -> sqlx::Result<Option<Tarea>> {
    todo!("SELECT por id con fetch_optional")
}

/// Primero las de prioridad alta; dentro de la misma prioridad, las más viejas primero.
pub async fn pagina(db: &PgPool, hecha: Option<bool>, pagina: i64) -> sqlx::Result<Vec<Tarea>> {
    todo!("filtro opcional por hecha, ORDER BY prioridad DESC, id, y LIMIT/OFFSET de POR_PAGINA")
}

/// Solo cambia los campos que llegan: COALESCE deja el valor actual si el nuevo es NULL.
pub async fn cambiar(db: &PgPool, id: i64, c: &Cambios) -> sqlx::Result<Option<Tarea>> {
    todo!("UPDATE con COALESCE para dejar lo que no llega")
}

pub async fn eliminar(db: &PgPool, id: i64) -> sqlx::Result<bool> {
    todo!("DELETE y true si borró una fila")
}

// ---------- Handlers: leen la petición, validan y llaman a las consultas ----------

pub async fn listar(
    State(db): State<PgPool>,
    filtro: Result<Query<Filtro>, QueryRejection>,
) -> Result<Json<Vec<Tarea>>, ApiError> {
    todo!("lee el filtro, rechaza página < 1 y llama a pagina()")
}

pub async fn crear(
    State(db): State<PgPool>,
    cuerpo: Result<Json<NuevaTarea>, JsonRejection>,
) -> Result<(StatusCode, Json<Tarea>), ApiError> {
    todo!("valida título y prioridad (2 si no llega), inserta y responde 201")
}

pub async fn obtener(
    State(db): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Tarea>, ApiError> {
    todo!("buscar() o 404")
}

pub async fn actualizar(
    State(db): State<PgPool>,
    Path(id): Path<i64>,
    cuerpo: Result<Json<Cambios>, JsonRejection>,
) -> Result<Json<Tarea>, ApiError> {
    todo!("valida lo que llegue y llama a cambiar(), o 404")
}

pub async fn borrar(State(db): State<PgPool>, Path(id): Path<i64>) -> Result<StatusCode, ApiError> {
    todo!("eliminar() y 204, o 404")
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_titulo_se_recorta_y_no_puede_ir_vacio() {
        assert_eq!(validar_titulo("  Comprar café ").unwrap(), "Comprar café");
        assert!(validar_titulo("").is_err());
        assert!(validar_titulo("   ").is_err());
    }

    #[test]
    fn el_titulo_cuenta_caracteres_no_bytes() {
        // 200 «ñ» son 400 bytes: deben pasar, porque el límite es de caracteres.
        assert!(validar_titulo(&"ñ".repeat(200)).is_ok());
        assert!(validar_titulo(&"ñ".repeat(201)).is_err());
    }

    #[test]
    fn la_prioridad_va_de_1_a_3() {
        for p in 1..=3 {
            assert_eq!(validar_prioridad(p).unwrap(), p);
        }
        assert!(validar_prioridad(0).is_err());
        assert!(validar_prioridad(4).is_err());
    }
}
