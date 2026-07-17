use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static ACTIVE_FILE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn set_active_file(path: &str) {
    if let Ok(mut guard) = ACTIVE_FILE.lock() {
        *guard = Some(path.to_string());
    }
}

pub fn get_active_file() -> Option<String> {
    ACTIVE_FILE.lock().ok()?.clone()
}
