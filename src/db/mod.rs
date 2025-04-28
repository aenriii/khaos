pub mod models;
pub mod schema;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use r2d2::Error as r2Error;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_pool() -> Result<DbPool, r2Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}
