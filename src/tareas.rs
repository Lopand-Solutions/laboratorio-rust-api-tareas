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
    let titulo = titulo.trim();
    if titulo.is_empty() {
        return Err(ApiError::Invalida("el título no puede ir vacío".into()));
    }
    if titulo.chars().count() > TITULO_MAXIMO {
        return Err(ApiError::Invalida(format!(
            "el título no puede pasar de {TITULO_MAXIMO} caracteres"
        )));
    }
    Ok(titulo.to_string())
}

pub fn validar_prioridad(prioridad: i16) -> Result<i16, ApiError> {
    if (1..=3).contains(&prioridad) {
        Ok(prioridad)
    } else {
        Err(ApiError::Invalida("la prioridad va de 1 a 3".into()))
    }
}

// ---------- Consultas: una función por operación, sin nada de HTTP ----------

pub async fn insertar(db: &PgPool, titulo: &str, prioridad: i16) -> sqlx::Result<Tarea> {
    sqlx::query_as!(
        Tarea,
        "INSERT INTO tareas (titulo, prioridad) VALUES ($1, $2)
         RETURNING id, titulo, hecha, prioridad",
        titulo,
        prioridad
    )
    .fetch_one(db)
    .await
}

pub async fn buscar(db: &PgPool, id: i64) -> sqlx::Result<Option<Tarea>> {
    sqlx::query_as!(
        Tarea,
        "SELECT id, titulo, hecha, prioridad FROM tareas WHERE id = $1",
        id
    )
    .fetch_optional(db)
    .await
}

/// Primero las de prioridad alta; dentro de la misma prioridad, las más viejas primero.
pub async fn pagina(db: &PgPool, hecha: Option<bool>, pagina: i64) -> sqlx::Result<Vec<Tarea>> {
    sqlx::query_as!(
        Tarea,
        "SELECT id, titulo, hecha, prioridad FROM tareas
         WHERE $1::bool IS NULL OR hecha = $1
         ORDER BY prioridad DESC, id
         LIMIT $2 OFFSET $3",
        hecha,
        POR_PAGINA,
        (pagina - 1) * POR_PAGINA
    )
    .fetch_all(db)
    .await
}

/// Solo cambia los campos que llegan: COALESCE deja el valor actual si el nuevo es NULL.
pub async fn cambiar(db: &PgPool, id: i64, c: &Cambios) -> sqlx::Result<Option<Tarea>> {
    sqlx::query_as!(
        Tarea,
        "UPDATE tareas SET
             titulo = COALESCE($2, titulo),
             hecha = COALESCE($3, hecha),
             prioridad = COALESCE($4, prioridad)
         WHERE id = $1
         RETURNING id, titulo, hecha, prioridad",
        id,
        c.titulo,
        c.hecha,
        c.prioridad
    )
    .fetch_optional(db)
    .await
}

pub async fn eliminar(db: &PgPool, id: i64) -> sqlx::Result<bool> {
    let r = sqlx::query!("DELETE FROM tareas WHERE id = $1", id)
        .execute(db)
        .await?;
    Ok(r.rows_affected() == 1)
}

// ---------- Handlers: leen la petición, validan y llaman a las consultas ----------

pub async fn listar(
    State(db): State<PgPool>,
    filtro: Result<Query<Filtro>, QueryRejection>,
) -> Result<Json<Vec<Tarea>>, ApiError> {
    let Query(filtro) = filtro?;
    let numero = filtro.pagina.unwrap_or(1);
    if numero < 1 {
        return Err(ApiError::Invalida("la página empieza en 1".into()));
    }
    Ok(Json(pagina(&db, filtro.hecha, numero).await?))
}

pub async fn crear(
    State(db): State<PgPool>,
    cuerpo: Result<Json<NuevaTarea>, JsonRejection>,
) -> Result<(StatusCode, Json<Tarea>), ApiError> {
    let Json(nueva) = cuerpo?;
    let titulo = validar_titulo(&nueva.titulo)?;
    let prioridad = validar_prioridad(nueva.prioridad.unwrap_or(2))?;
    let tarea = insertar(&db, &titulo, prioridad).await?;
    Ok((StatusCode::CREATED, Json(tarea)))
}

pub async fn obtener(
    State(db): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Tarea>, ApiError> {
    buscar(&db, id)
        .await?
        .map(Json)
        .ok_or(ApiError::NoEncontrada(id))
}

pub async fn actualizar(
    State(db): State<PgPool>,
    Path(id): Path<i64>,
    cuerpo: Result<Json<Cambios>, JsonRejection>,
) -> Result<Json<Tarea>, ApiError> {
    let Json(mut cambios) = cuerpo?;
    if let Some(t) = &cambios.titulo {
        cambios.titulo = Some(validar_titulo(t)?);
    }
    if let Some(p) = cambios.prioridad {
        validar_prioridad(p)?;
    }
    cambiar(&db, id, &cambios)
        .await?
        .map(Json)
        .ok_or(ApiError::NoEncontrada(id))
}

pub async fn borrar(State(db): State<PgPool>, Path(id): Path<i64>) -> Result<StatusCode, ApiError> {
    if eliminar(&db, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NoEncontrada(id))
    }
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
