use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.
			alter_table(
				Table::alter()
					.table(Panels::Table)
					.drop_column(Panels::Width)
					.drop_column(Panels::LimitView)
					.drop_column(Panels::ShiftView)
					.to_owned()
			)
			.await?;
		manager.
			alter_table(
				Table::alter()
					.table(Metrics::Table)
					.drop_column(Metrics::PanelId)
					.to_owned()
			)
			.await?;
		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Panels::Table)
					.add_column(
						ColumnDef::new(Panels::Width)
							.integer()
							.not_null()
							.default(100)
					)
					.add_column(
						ColumnDef::new(Panels::LimitView)
							.boolean()
							.not_null()
							.default(true)
					)
					.add_column(
						ColumnDef::new(Panels::ShiftView)
							.boolean()
							.not_null()
							.default(false)
					)
					.to_owned()
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(Metrics::Table)
					.add_column(
						ColumnDef::new(Metrics::PanelId)
							.big_integer()
							.not_null()
							.default(0)
					)
					.to_owned()
			)
			.await?;
		Ok(())
	}
}

#[derive(Iden)]
enum Panels {
	Table,
	Width,
	LimitView,
	ShiftView,
}

#[derive(Iden)]
enum Metrics {
	Table,
	PanelId,
}
