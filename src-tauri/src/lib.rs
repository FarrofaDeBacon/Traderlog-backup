mod commands;
mod db;
mod hardware;
mod models;
mod seed;
mod logic;
mod services;
#[cfg(test)]
mod logic_tests;

use crate::db::DbState;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

static RTD_MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);
static RTD_CHILD_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);

#[tauri::command]
fn start_rtd_monitor_cmd(app_handle: AppHandle, excel_path: Option<String>) {
    start_rtd_monitor(app_handle, excel_path);
}

fn start_rtd_monitor(app_handle: AppHandle, excel_path: Option<String>) {
    #[cfg(target_os = "windows")]
    {
        println!(
            "[RTD] Initializing monitor request. Excel path: {:?}",
            excel_path
        );

        // 1. Start CSV Monitor Thread (ONLY ONCE across app lifecycle)
        if !RTD_MONITOR_RUNNING.load(Ordering::SeqCst) {
            let app_handle_clone = app_handle.clone();
            let spawn_result = std::thread::Builder::new()
                .name("rtd-csv-monitor".into())
                .spawn(move || {
                    let temp_path =
                        std::env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
                    let csv_path = PathBuf::from(temp_path).join("traderlog_rtd_data.csv");

                    println!(
                        "[RTD] CSV Monitor thread started successfully at {:?}",
                        csv_path
                    );
                    loop {
                        if csv_path.exists() {
                            // Resilient read: try multiple times if it's locked by another process (common on Windows)
                            let mut content = String::new();
                            let mut success = false;
                            for _ in 0..3 {
                                if let Ok(c) = fs::read_to_string(&csv_path) {
                                    content = c;
                                    success = true;
                                    break;
                                }
                                std::thread::sleep(Duration::from_millis(100));
                            }

                            if success && !content.trim().is_empty() {
                                app_handle_clone.emit("rtd-update", content).ok();
                            }
                        }
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                });

            match spawn_result {
                Ok(_) => {
                    RTD_MONITOR_RUNNING.store(true, Ordering::SeqCst);
                    println!("[RTD] Monitor flag set to true.");
                }
                Err(e) => {
                    println!(
                        "[RTD] CRITICAL ERROR: Could not spawn CSV monitor thread: {}",
                        e
                    );
                }
            }
        }

        // 2. Kill old process if tracked and start new one
        if let Ok(mut lock) = RTD_CHILD_PROCESS.lock() {
            if let Some(mut child) = lock.take() {
                println!("[RTD] Terminating existing bridge process...");
                let _ = child.kill().ok();
                // Removed wait() to prevent hanging the setup sequence
            }
        }

        // Auto-open Excel file if path is provided
        if let Some(ref path) = excel_path {
            if !path.trim().is_empty() {
                println!("[RTD] Triggering auto-open for Excel: {}", path);
                Command::new("cmd")
                    .args(&["/C", "start", "", path])
                    .spawn()
                    .ok();
            }
        }

        let resource_dir = app_handle.path().resource_dir().unwrap_or_default();
        let script_path = resource_dir.join("scripts").join("rtd_bridge.ps1");

        // Find script path (dev vs prod)
        let script_final = if script_path.exists() {
            script_path
        } else {
            let p1 = PathBuf::from("src-tauri/scripts/rtd_bridge.ps1");
            let p2 = PathBuf::from("scripts/rtd_bridge.ps1");
            if p1.exists() {
                p1
            } else if p2.exists() {
                p2
            } else {
                script_path
            }
        };

        if script_final.exists() {
            println!("[RTD] Spawning PowerShell bridge: {:?}", script_final);
            let mut args = vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-WindowStyle".to_string(),
                "Hidden".to_string(),
                "-File".to_string(),
                script_final.to_str().unwrap().to_string(),
            ];

            if let Some(path) = excel_path {
                if !path.trim().is_empty() {
                    args.push("-ExcelPath".to_string());
                    args.push(path);
                }
            }

            match Command::new("powershell.exe").args(&args).spawn() {
                Ok(child) => {
                    println!("[RTD] Bridge process spawned (PID: {})", child.id());
                    if let Ok(mut lock) = RTD_CHILD_PROCESS.lock() {
                        *lock = Some(child);
                    }
                }
                Err(e) => {
                    println!("[RTD] FAILED to spawn bridge process: {}", e);
                }
            }
        } else {
            println!("[RTD] ERROR: Bridge script NOT FOUND at {:?}", script_final);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            println!("[SETUP] Starting app setup...");

            let handle = app.handle().clone();

            // Initialize Database
            let db_handle = handle.clone();
            tauri::async_runtime::block_on(async move {
                println!("[STARTUP] Initializing Database...");
                match db::init_db(&db_handle).await {
                    Ok(db) => {
                        println!(
                            "[STARTUP] Database initialized. Running Base Seeds (UPSERT mode)..."
                        );
                        if let Err(e) = seed::run_base_seeds(&db).await {
                            println!("[STARTUP] SEED ERROR: {}", e);
                        }

                        println!("[STARTUP] Registering Database state.");
                        db_handle.manage(DbState(db));
                    }
                    Err(e) => {
                        println!("[STARTUP] FATAL DATABASE ERROR: {}", e);
                        // Do not panic, just log. App might show blank but won't crash process.
                    }
                }
            });

            println!("[SETUP] Initial setup flow completed.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_rtd_monitor_cmd,
            commands::get_user_profile,
            commands::save_user_profile,
            commands::verify_password,
            commands::verify_recovery_key,
            commands::reset_password,
            commands::get_api_configs,
            commands::save_api_config,
            commands::get_accounts,
            commands::save_account,
            commands::delete_account,
            commands::get_currencies,
            commands::save_currency,
            commands::get_markets,
            commands::save_market,
            commands::get_asset_types,
            commands::save_asset_type,
            commands::get_assets,
            commands::save_asset,
            commands::get_emotional_states,
            commands::save_emotional_state,
            commands::seed_emotional_states,
            commands::get_strategies,
            commands::save_strategy,
            // Entidades Novas
            commands::get_trades,
            commands::save_trade,
            commands::delete_trade,
            commands::delete_trades_by_ids,
            commands::get_cash_transactions,
            commands::save_cash_transaction,
            commands::delete_cash_transaction,
            commands::get_journal_entries,
            commands::save_journal_entry,
            commands::delete_journal_entry,
            commands::get_fees,
            commands::save_fee,
            commands::delete_fee,
            commands::get_risk_profiles,
            commands::save_risk_profile,
            commands::delete_risk_profile,
            commands::get_modalities,
            commands::save_modality,
            commands::delete_modality,
            commands::get_tags,
            commands::save_tag,
            commands::delete_tag,
            commands::get_indicators,
            commands::save_indicator,
            commands::delete_indicator,
            commands::get_timeframes,
            commands::save_timeframe,
            commands::delete_timeframe,
            commands::get_chart_types,
            commands::save_chart_type,
            commands::delete_chart_type,
            commands::delete_market,
            commands::delete_asset_type,
            commands::delete_asset,
            commands::delete_currency,
            commands::delete_strategy,
            commands::delete_emotional_state,
            commands::force_reseed,
            commands::ensure_defaults,
            commands::check_database_status,
            commands::seed_demo_account,
            commands::delete_demo_account_data,
            commands::delete_all_demo_trades,
            commands::seed_demo_data,
            commands::seed_stress_data,
            commands::complete_onboarding,
            commands::finish_custom_onboarding,
            commands::get_onboarding_meta,
            commands::get_machine_id_cmd,
            commands::validate_license_cmd,
            commands::factory_reset,
            commands::backup_database,
            commands::restore_database,
            // Fiscal / IRPF
            commands::irpf::calculate_monthly_tax,
            commands::irpf::get_appraisals,
            commands::irpf::save_appraisal,
            commands::irpf::get_accumulated_losses,
            commands::irpf::delete_appraisal,
            commands::irpf::generate_darf,
            commands::irpf::get_darfs,
            commands::irpf::mark_darf_paid,
            commands::irpf::unpay_darf,
            commands::irpf::delete_darf,
            commands::irpf::calculate_interest_fine,
            commands::irpf::reset_irpf_data,
            commands::irpf::delete_tax_loss,
            commands::irpf::get_darf_by_transaction,
            commands::irpf::get_darf_by_id,
            // Tax Rules & Mappings
            commands::irpf::get_tax_rules,
            commands::irpf::save_tax_rule,
            commands::irpf::delete_tax_rule,
            commands::irpf::get_tax_mappings,
            commands::irpf::save_tax_mapping,
            commands::irpf::delete_tax_mapping,
            // Tax Profiles
            commands::irpf::get_tax_profiles,
            commands::irpf::save_tax_profile,
            commands::irpf::delete_tax_profile,
            commands::irpf::get_tax_profile_entries,
            commands::irpf::save_tax_profile_entry,
            commands::irpf::delete_tax_profile_entry,
            commands::irpf::diagnostic_dump_darfs,
            commands::irpf::diagnostic_fix_darf_status,
            commands::irpf::get_appraisal_by_id,
            commands::diagnostic_dump_users,
            commands::diagnostic_dump_trades,
            commands::diagnostic_closure_dump,
            commands::open_detached_trade_window,
            commands::profit_import::import_profit_trades,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
