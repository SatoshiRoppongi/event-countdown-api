// src/bin/reset_dev.rs

use diesel::prelude::*;
use std::fs;
use std::env;
use std::path::Path;

fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let mut conn = PgConnection::establish(&database_url)
        .expect("Failed to connect to database");

    let sql_path = Path::new("sql/reset_dev.sql");

    let sql = fs::read_to_string(sql_path)
        .expect("Could not read sql/reset_dev.sql");


    // SQL文をセミコロンで分割し、空白除去してから実行
    for statement in sql.split(';') {
      let statement = statement.trim();
      if statement.is_empty() {
        continue;
      }
      diesel::sql_query(statement).execute(&mut conn).expect(&format!("Failed to execute statement: {}", statement));
    }
    println!("✅ Development database reset and seeded successfully.");
}
