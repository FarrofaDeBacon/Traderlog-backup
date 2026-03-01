use std::path::PathBuf;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;
use tauri::AppHandle;
use tauri::Manager;

pub struct DbState(pub Surreal<Db>);

pub async fn init_db(app_handle: &AppHandle) -> Result<Surreal<Db>, surrealdb::Error> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    // Create directory if not exists
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
    }

    let db_path = app_dir.join("traderlog.db");
    println!("[DB] Attempting to open database at: {:?}", db_path);

    let db = Surreal::new::<SurrealKv>(db_path.to_str().unwrap()).await?;
    println!("[DB] SurrealDB instance created successfully.");

    db.use_ns("traderlog").use_db("main").await?;
    println!("[DB] Namespace 'traderlog' and Database 'main' set successfully.");

    Ok(db)
}
