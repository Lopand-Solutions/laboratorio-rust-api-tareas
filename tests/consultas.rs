//! Pruebas de las consultas contra un Postgres de verdad.
//! #[sqlx::test] crea una base nueva para cada prueba, le aplica las migraciones y la borra al final.
use api_tareas::tareas::{self, Cambios, POR_PAGINA};
use sqlx::PgPool;

#[sqlx::test]
async fn crea_y_busca_una_tarea(db: PgPool) {
    let creada = tareas::insertar(&db, "Comprar café", 2).await.unwrap();
    assert!(!creada.hecha);
    let leida = tareas::buscar(&db, creada.id).await.unwrap();
    assert_eq!(leida, Some(creada));
    assert_eq!(tareas::buscar(&db, 9999).await.unwrap(), None);
}

#[sqlx::test]
async fn ordena_por_prioridad_y_filtra_por_hecha(db: PgPool) {
    let baja = tareas::insertar(&db, "baja", 1).await.unwrap();
    let alta = tareas::insertar(&db, "alta", 3).await.unwrap();
    let media = tareas::insertar(&db, "media", 2).await.unwrap();
    let cambio = Cambios {
        hecha: Some(true),
        ..Default::default()
    };
    tareas::cambiar(&db, media.id, &cambio).await.unwrap();

    let todas = tareas::pagina(&db, None, 1).await.unwrap();
    let titulos: Vec<_> = todas.iter().map(|t| t.titulo.as_str()).collect();
    assert_eq!(titulos, ["alta", "media", "baja"]);

    let pendientes = tareas::pagina(&db, Some(false), 1).await.unwrap();
    let ids: Vec<_> = pendientes.iter().map(|t| t.id).collect();
    assert_eq!(ids, [alta.id, baja.id]);
}

#[sqlx::test]
async fn pagina_de_veinte_en_veinte(db: PgPool) {
    for i in 1..=25 {
        tareas::insertar(&db, &format!("tarea {i}"), 2)
            .await
            .unwrap();
    }
    let primera = tareas::pagina(&db, None, 1).await.unwrap();
    let segunda = tareas::pagina(&db, None, 2).await.unwrap();
    assert_eq!(primera.len() as i64, POR_PAGINA);
    assert_eq!(segunda.len(), 5);
    assert_eq!(primera[0].titulo, "tarea 1");
    assert_eq!(segunda[0].titulo, "tarea 21");
}

#[sqlx::test]
async fn cambiar_solo_toca_lo_que_llega(db: PgPool) {
    let t = tareas::insertar(&db, "Llamar a Luis", 1).await.unwrap();
    let cambio = Cambios {
        hecha: Some(true),
        ..Default::default()
    };
    let nueva = tareas::cambiar(&db, t.id, &cambio).await.unwrap().unwrap();
    assert_eq!(nueva.titulo, "Llamar a Luis");
    assert_eq!(nueva.prioridad, 1);
    assert!(nueva.hecha);
    assert_eq!(tareas::cambiar(&db, 9999, &cambio).await.unwrap(), None);
}

#[sqlx::test]
async fn eliminar_dice_si_habia_algo(db: PgPool) {
    let t = tareas::insertar(&db, "temporal", 2).await.unwrap();
    assert!(tareas::eliminar(&db, t.id).await.unwrap());
    assert!(!tareas::eliminar(&db, t.id).await.unwrap());
}
