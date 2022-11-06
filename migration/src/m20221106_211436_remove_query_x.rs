use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Metrics::Table)
					.drop_column(Metrics::QueryX)
					.to_owned()
			).await?;

		manager
			.alter_table(
				Table::alter()
					.table(Metrics::Table)
					.rename_column(Metrics::QueryY, Metrics::Query)
					.to_owned()
		).await?;

		manager
			.alter_table(
				Table::alter()
					.table(Panels::Table)
					.drop_column(Panels::Timeserie)
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Metrics::Table)
					.rename_column(Metrics::Query, Metrics::QueryY)
					.to_owned()
		).await?;

		manager.
			alter_table(
				Table::alter()
					.table(Metrics::Table)
					.add_column(
						ColumnDef::new(Metrics::QueryX)
							.float()
							.not_null()
							.default(0.0)
					)
					.to_owned()
			).await?;

		manager.
			alter_table(
				Table::alter()
					.table(Panels::Table)
					.add_column(
						ColumnDef::new(Panels::Timeserie)
							.boolean()
							.not_null()
							.default(true)
					)
					.to_owned()
			).await?;

		Ok(())
	}
}

#[derive(Iden)]
enum Metrics {
	Table,
	QueryX,
	QueryY,
	Query,
}

#[derive(Iden)]
enum Panels {
	Table,
	Timeserie,
}
