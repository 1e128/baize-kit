fn main() {
    // 加载 .env
    if let Err(err) = dotenvy::from_path(".env") {
        eprintln!("Warning: failed to load .env: {}", err);
    }

    // 确保环境变量传递给编译期宏（如 sqlx::query!）
    println!("cargo:rerun-if-changed=.env");
}
