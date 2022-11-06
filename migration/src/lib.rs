pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20221030_192706_add_last_update;
mod m20221102_232244_add_join_table;
mod m20221102_232858_remove_unused_columns;
mod m20221106_211436_remove_query_x;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20221030_192706_add_last_update::Migration),
            Box::new(m20221102_232244_add_join_table::Migration),
            Box::new(m20221102_232858_remove_unused_columns::Migration),
            Box::new(m20221106_211436_remove_query_x::Migration),
        ]
	}
}
