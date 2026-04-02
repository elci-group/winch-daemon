pub fn notify_success(message: &str) {
    println!("✅ {}", message);
}

pub fn notify_warning(message: &str) {
    println!("⚠️ {}", message);
}

pub fn notify_error(message: &str) {
    eprintln!("❌ {}", message);
}
