// src-tauri/src/seed/tax_seed.rs
use crate::models::irpf::{TaxAppraisal, TaxDarf};
use crate::models::{TaxProfile, TaxProfileEntry, TaxRule};
use chrono::{Datelike, Utc};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_tax_rules(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Verificando Regras e Perfis Tributários (B3)...");

    // 1. Tax Rules
    let rules = vec![
        (
            "rule_swing_acoes",
            "Swing Trade Ações (15%)",
            15.0,
            0.005,
            20000.0,
            "NetProfit",
            true,
            "SwingTrade",
            "SalesVolume",
            "6015",
        ),
        (
            "rule_day_acoes",
            "Day Trade Ações (20%)",
            20.0,
            1.0,
            0.0,
            "NetProfit",
            true,
            "DayTrade",
            "Profit",
            "6015",
        ),
        (
            "rule_swing_futuros",
            "Swing Trade Futuros (15%)",
            15.0,
            0.005,
            0.0,
            "NetProfit",
            true,
            "SwingTrade",
            "Profit",
            "6015",
        ),
        (
            "rule_day_futuros",
            "Day Trade Futuros (20%)",
            20.0,
            1.0,
            0.0,
            "NetProfit",
            true,
            "DayTrade",
            "Profit",
            "6015",
        ),
        (
            "rule_fiis",
            "FIIs (20%)",
            20.0,
            0.0,
            0.0,
            "NetProfit",
            true,
            "SwingTrade",
            "Profit",
            "6015",
        ),
    ];

    for (id, name, rate, w_rate, exemption, basis, cumulative, t_type, w_basis, rev_code) in rules {
        let rule = TaxRule {
            id: Some(id.into()),
            name: name.into(),
            tax_rate: rate,
            withholding_rate: w_rate,
            exemption_threshold: exemption,
            basis: basis.into(),
            cumulative_losses: cumulative,
            trade_type: t_type.into(),
            withholding_basis: w_basis.into(),
            revenue_code: Some(rev_code.into()),
            metadata: std::collections::HashMap::new(),
        };

        let mut data = serde_json::to_value(&rule).unwrap();
        if let Some(obj) = data.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('tax_rule', $id) CONTENT $data RETURN NONE")
            .bind(("id", id))
            .bind(("data", data))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ Rule: {}", name);
    }

    // 2. Tax Profiles
    let profiles = vec![
        (
            "tp_acoes",
            "Perfil Tributário Ações",
            "Tributação padrão de ações (Isenção 20k no Swing)",
        ),
        (
            "tp_futuros",
            "Perfil Tributário Futuros",
            "Tributação de Índices e Dólar (Sem isenção)",
        ),
    ];

    for (id, name, desc) in profiles {
        let profile = TaxProfile {
            id: Some(id.into()),
            name: name.into(),
            description: Some(desc.into()),
            metadata: std::collections::HashMap::new(),
        };
        let mut data = serde_json::to_value(&profile).unwrap();
        if let Some(obj) = data.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('tax_profile', $id) CONTENT $data RETURN NONE")
            .bind(("id", id))
            .bind(("data", data))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ Profile: {}", name);
    }

    // 3. Tax Profile Entries (Linking Profile -> Modality -> Rule)
    let entries = vec![
        (
            "tpe_acoes_swing",
            "tax_profile:tp_acoes",
            "modality:mod2",
            "tax_rule:rule_swing_acoes",
        ),
        (
            "tpe_acoes_day",
            "tax_profile:tp_acoes",
            "modality:mod1",
            "tax_rule:rule_day_acoes",
        ),
        (
            "tpe_futuros_swing",
            "tax_profile:tp_futuros",
            "modality:mod2",
            "tax_rule:rule_swing_futuros",
        ),
        (
            "tpe_futuros_day",
            "tax_profile:tp_futuros",
            "modality:mod1",
            "tax_rule:rule_day_futuros",
        ),
    ];

    for (id, p_id, m_id, r_id) in entries {
        let entry = TaxProfileEntry {
            id: Some(id.into()),
            tax_profile_id: Some(p_id.into()),
            modality_id: Some(m_id.into()),
            tax_rule_id: Some(r_id.into()),
            metadata: std::collections::HashMap::new(),
        };
        let mut data = serde_json::to_value(&entry).unwrap();
        if let Some(obj) = data.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('tax_profile_entry', $id) CONTENT $data RETURN NONE")
            .bind(("id", id))
            .bind(("data", data))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ Entry: {}", id);
    }

    println!("[SEED] ✅ Regras e Perfis Tributários sincronizados.");

    // (Os registros de impostos de demonstração agora são gerados separadamente em run_all_seeds)

    Ok(())
}

pub async fn seed_initial_tax_records(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Gerando Registros de Impostos de Demonstração...");

    let now = Utc::now();
    let current_year = now.year() as u16;
    let current_month = now.month() as u8;

    // 1. Tax Appraisals (Jan 2026 - Assuming current year is 2026 in demo)
    let appraisals = vec![
        (
            1,
            2026,
            "SwingTrade",
            "Swing Trade Ações (15%)",
            "6015",
            5000.0,
            0.0,
            5000.0,
            0.0,
            5000.0,
            15.0,
            750.0,
            25.0,
            725.0,
            "tax_rule:rule_swing_acoes",
        ),
        (
            1,
            2026,
            "DayTrade",
            "Day Trade Futuros (20%)",
            "6015",
            2500.0,
            500.0,
            2000.0,
            0.0,
            2000.0,
            20.0,
            400.0,
            10.0,
            390.0,
            "tax_rule:rule_day_futuros",
        ),
        (
            12,
            2025,
            "SwingTrade",
            "Swing Trade Ações (15%)",
            "6015",
            3000.0,
            0.0,
            3000.0,
            0.0,
            3000.0,
            15.0,
            450.0,
            15.0,
            435.0,
            "tax_rule:rule_swing_acoes",
        ),
    ];

    for (
        m,
        y,
        t_type,
        _r_name,
        r_code,
        gross,
        loss,
        net,
        comp,
        basis,
        rate,
        due,
        withheld,
        payable,
        rule_id,
    ) in appraisals
    {
        let id = format!("tax_appraisal:demo_{}_{}_{}", y, m, t_type.to_lowercase());

        let appraisal = TaxAppraisal {
            id: Some(id.clone()),
            period_month: m,
            period_year: y,
            trade_type: t_type.into(),
            tax_rule_id: Some(rule_id.into()),
            revenue_code: Some(r_code.into()),
            gross_profit: gross,
            loss,
            net_profit: net,
            compensated_loss: comp,
            calculation_basis: basis,
            tax_rate: rate,
            tax_due: due,
            withheld_tax: withheld,
            withholding_credit_used: 0.0,
            withholding_credit_remaining: 0.0,
            tax_payable: payable,
            tax_accumulated: 0.0,
            total_payable: payable,
            is_exempt: false,
            calculation_date: Utc::now().to_rfc3339(),
            status: if y < current_year || (y == current_year && m < current_month) {
                "Paid".into()
            } else {
                "Pending".into()
            },
            trade_ids: vec![],
            is_complementary: false,
            metadata: std::collections::HashMap::new(),
        };

        let mut data = serde_json::to_value(&appraisal).unwrap();
        if let Some(obj) = data.as_object_mut() {
            obj.remove("id");
        }

        db.query("UPSERT type::thing('tax_appraisal', $id) CONTENT $data RETURN NONE")
            .bind(("id", id.split(':').last().unwrap().to_string()))
            .bind(("data", data))
            .await
            .map_err(|e| e.to_string())?;

        // 2. DARF (if payable > 0 and Status is Paid or Pending)
        if payable > 0.0 {
            let darf_id = format!("tax_darf:demo_{}_{}_{}", y, m, t_type.to_lowercase());
            let due_date = format!(
                "{}-{:02}-28T23:59:59Z",
                if m == 12 { y + 1 } else { y },
                if m == 12 { 1 } else { m + 1 }
            );

            let darf = TaxDarf {
                id: Some(darf_id.clone()),
                appraisal_id: Some(id.clone()),
                revenue_code: Some(r_code.into()),
                period: format!("{:02}/{}", m, y),
                principal_value: payable,
                fine: 0.0,
                interest: 0.0,
                total_value: payable,
                due_date,
                payment_date: if y < current_year || (y == current_year && m < current_month) {
                    Some(Utc::now().to_rfc3339())
                } else {
                    None
                },
                status: if y < current_year || (y == current_year && m < current_month) {
                    "Paid".into()
                } else {
                    "Pending".into()
                },
                darf_number: Some(format!("{:010}", 1234567890 + m as i64)),
                account_id: Some("account:demo_b3_futuros".into()),
                transaction_id: None,
                is_complementary: false,
                metadata: std::collections::HashMap::new(),
            };

            let mut darf_data = serde_json::to_value(&darf).unwrap();
            if let Some(obj) = darf_data.as_object_mut() {
                obj.remove("id");
            }

            db.query("UPSERT type::thing('tax_darf', $id) CONTENT $data RETURN NONE")
                .bind(("id", darf_id.split(':').last().unwrap().to_string()))
                .bind(("data", darf_data))
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    println!("  ✓ Registros de impostos de demonstração criados.");
    Ok(())
}
