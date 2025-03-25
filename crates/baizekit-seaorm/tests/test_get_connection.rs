use std::env;

use baizekit_seaorm::*;

#[tokio::test]
async fn test_get_connection() {
    unsafe {
        env::set_var("DATABASE_URL", "postgres://root:123456@localhost:5432/postgres");
    }

    let conn = try_get_database_connection().await;
    println!("conn: {:?}", conn);
}
