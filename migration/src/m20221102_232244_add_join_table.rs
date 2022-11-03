use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(PanelMetric::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(PanelMetric::Id)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(PanelMetric::PanelId).big_integer().not_null())
					.col(ColumnDef::new(PanelMetric::MetricId).big_integer().not_null())
					.to_owned(),
			).await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(PanelMetric::Table).to_owned())
			.await
	}
}

#[derive(Iden)]
enum PanelMetric {
	Table,
	Id,
	PanelId,
	MetricId,
}
