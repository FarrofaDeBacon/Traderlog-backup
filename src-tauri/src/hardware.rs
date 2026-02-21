use std::process::Command;

pub fn get_machine_id() -> String {
    // Standardize on machine-uid crate for speed and reliability.
    // WMI/PowerShell calls can hang the setup sequence on some systems.
    match machine_uid::get() {
        Ok(uid) => uid,
        Err(_) => {
            // Last resort: standard fallback
            "UNKNOWN_HWID".to_string()
        }
    }
}
