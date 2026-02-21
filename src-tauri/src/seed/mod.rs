// src-tauri/src/seed/mod.rs

pub mod _master_seeder;
pub mod asset_types_seed;
pub mod assets_seed;
pub mod chart_types_seed;
pub mod currencies_seed;
pub mod defaults;
pub mod demo_accounts_seed;
pub mod demo_trades_seed;
pub mod emotional_states_seed;
pub mod fees_seed;
pub mod indicators_seed;
pub mod markets_seed;
pub mod modalities_seed;
pub mod risk_seed;
pub mod strategies_seed;
pub mod tags_seed;
pub mod tax_seed;
pub mod timeframes_seed;
pub mod user_profile_seed;

// Expõe publicamente as funções que serão chamadas de fora
pub use _master_seeder::force_reseed_all;
pub use _master_seeder::run_all_seeds;
pub use _master_seeder::run_minimal_seeds;
pub use _master_seeder::run_selective_seeds;
