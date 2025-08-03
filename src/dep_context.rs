use crate::config::{Config, get_figment_for_other};
use crate::db::SqliteClient;
use rocket::fairing::AdHoc;
use std::sync::Arc;

pub struct DepContext {
    #[allow(dead_code)]
    config: Arc<Config>,
    #[allow(dead_code)]
    sqlite_client: SqliteClient,
}

impl DepContext {
    pub fn adhoc() -> AdHoc {
        AdHoc::on_ignite("DepContext", |rocket| async {
            let config = get_figment_for_other()
                .extract::<Arc<Config>>()
                .expect("Failed to extract config");

            let sqlite_client =
                SqliteClient::new(config.sqlite_path.clone()).expect("Failed to connect to sqlite");

            let dep_context = DepContext {
                config,
                sqlite_client,
            };

            rocket.manage(dep_context)
        })
    }
}
