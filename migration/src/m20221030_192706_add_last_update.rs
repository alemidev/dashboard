use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.
			alter_table(
				Table::alter()
					.table(Sources::Table)
					.add_column(
						ColumnDef::new(Sources::LastUpdate)
							.big_integer()
							.not_null()
							.default(0)
					)
					.to_owned()
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Sources::Table)
					.drop_column(Sources::LastUpdate)
					.to_owned()
			)
			.await
	}
}

#[derive(Iden)]
enum Sources {
	Table,
	LastUpdate,
}
