use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

pub fn get_connection() -> Result<PgConnection, diesel::ConnectionError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
}
