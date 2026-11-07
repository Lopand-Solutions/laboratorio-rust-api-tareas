# Laboratorio: una API de tareas con Axum y Postgres

Repositorio del laboratorio de [Lopand Academy](https://academy.lopandsolutions.com/tutoriales/laboratorio-rust-api-tareas).
Una API REST de tareas con Axum 0.8 y SQLx 0.9 sobre Postgres: crear, listar con filtro y páginas, cambiar solo lo
que llega y borrar. Con pruebas de validación sin base, pruebas de consultas con `#[sqlx::test]` y el reto
«rómpelo y arréglalo».

## Ramas

- **`inicio`**: el proyecto armado (migración, errores, router, `main.rs`) y las pruebas. En `src/tareas.rs` las
  funciones tienen `todo!()`: ahí escribes la validación, las consultas y los handlers.
- **`main`**: la solución, tal como está en la lección, con la carpeta `.sqlx` para compilar sin base.

```bash
git clone -b inicio https://github.com/Lopand-Solutions/laboratorio-rust-api-tareas.git api-tareas
cd api-tareas
cp .env.ejemplo .env   # y pon ahí la URL de tu Postgres
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx database create
sqlx migrate run
```

## Córrelo

```bash
cargo test
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/tareas
cargo run
```

`cargo test` lee `DATABASE_URL` del `.env`; `cargo run` no, por eso el `export`.

## Cómo saber si quedó

```bash
./tool/comprobar_laboratorio.sh
```

Revisa el formato, corre clippy **sin base de datos** (con la carpeta `.sqlx`), corre las pruebas de validación y
las de consultas, rompe tu código en una copia (los tres retos de la lección) para ver que tus pruebas lo detectan y
revisa que `.env` no entre al repositorio. Necesita un Postgres para las pruebas de consultas y los retos.

Cuando cambies una consulta, actualiza el caché offline:

```bash
cargo sqlx prepare -- --all-targets
```

## Requisitos

Rust 1.94 o superior (lo pide SQLx 0.9), Postgres y `sqlx-cli`. Las versiones de `Cargo.toml` son las que probamos.

Código libre para usar y adaptar (licencia MIT).
