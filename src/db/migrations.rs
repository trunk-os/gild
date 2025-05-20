use welds::errors::Result;
use welds::migrations::prelude::*;

use super::DB;

//
// boilerplate for migration execution
//

static MIGRATIONS: &[fn(&TableState) -> Result<MigrationStep>] =
    &[create_users_table, create_sessions_table];

pub(crate) async fn migrate(db: &DB) -> Result<()> {
    Ok(up(&db.0, MIGRATIONS).await?)
}

//
// actual migrations, in order
//
// to add a migration, create your function at the end and then append it to the list of MIGRATIONS
// near the top.
//

fn create_users_table(_: &TableState) -> Result<MigrationStep> {
    let m = create_table("users")
        .id(|c| c("id", Type::Uuid))
        .column(|c| c("username", Type::String))
        .column(|c| c("password", Type::String))
        .column(|c| c("realname", Type::String).is_null())
        .column(|c| c("email", Type::String).is_null())
        .column(|c| c("phone", Type::String).is_null());
    Ok(MigrationStep::new("create users table", m))
}

fn create_sessions_table(_: &TableState) -> Result<MigrationStep> {
    let m = create_table("sessions")
        .id(|c| c("id", Type::Uuid))
        .column(|c| c("secret", Type::String))
        .column(|c| c("expires", Type::Datetime));
    Ok(MigrationStep::new("create sessions table", m))
}
