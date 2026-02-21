// src-tauri/src/seed/user_profile_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::UserProfile;

pub async fn seed_user_profile(db: &Surreal<Db>) -> Result<(), String> {
    // Check if user profile already exists - Use direct record ID and cast to string
    let mut result = db.query("SELECT *, type::string(id) as id FROM user_profile:main").await.map_err(|e| e.to_string())?;
    let profile: Vec<UserProfile> = result.take(0).map_err(|e| e.to_string())?;
    
    if !profile.is_empty() {
        println!("[SEED] Perfil de Usuário já existe. Pulando.");
        return Ok(());
    }

    println!("[SEED] Criando Perfil de Usuário Padrão...");

    let profile_data = UserProfile {
        id: "main".into(),
        name: "Trader".into(),
        email: "".into(),
        phone: "".into(),
        cpf: "".into(),
        theme: "dark".into(),
        language: "pt-BR".into(),
        timezone: "America/Sao_Paulo".into(),
        main_currency: "BRL".into(),
        avatar: None,
        convert_all_to_main: false,
        onboarding_completed: false,
        currency_api_url: Some("https://economia.awesomeapi.com.br/last/".into()),
        birth_date: None,
        trial_start_date: None,
        license_key: None,
        utc_offset: -180,
    };
    let mut json_data = serde_json::to_value(&profile_data).unwrap();
    if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

    db.query("CREATE user_profile:main CONTENT $data")
        .bind(("data", json_data))
        .await
        .map_err(|e| e.to_string())?;
    println!("  ✓ Perfil Padrão");

    Ok(())
}
