use sea_orm_migration::prelude::*;


// I wish I had used SeaOrm since the beginning:
//  this first migration wouldn't be so beefy!

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Panels::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Panels::Id)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Panels::Name).string().not_null())
					.col(ColumnDef::new(Panels::Position).integer().not_null())
					.col(ColumnDef::new(Panels::Timeserie).boolean().not_null())
					.col(ColumnDef::new(Panels::Height).integer().not_null())
					.col(ColumnDef::new(Panels::Width).integer().not_null())
					.col(ColumnDef::new(Panels::ViewScroll).boolean().not_null())
					.col(ColumnDef::new(Panels::LimitView).boolean().not_null())
					.col(ColumnDef::new(Panels::ViewSize).integer().not_null())
					.col(ColumnDef::new(Panels::ReduceView).boolean().not_null())
					.col(ColumnDef::new(Panels::ViewChunks).integer().not_null())
					.col(ColumnDef::new(Panels::ShiftView).boolean().not_null())
					.col(ColumnDef::new(Panels::ViewOffset).integer().not_null())
					.col(ColumnDef::new(Panels::AverageView).boolean().not_null())
					.to_owned(),
			).await?;
		manager
			.create_table(
				Table::create()
					.table(Sources::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Sources::Id)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Sources::Name).string().not_null())
					.col(ColumnDef::new(Sources::Position).integer().not_null())
					.col(ColumnDef::new(Sources::Enabled).boolean().not_null())
					.col(ColumnDef::new(Sources::Url).string().not_null())
					.col(ColumnDef::new(Sources::Interval).integer().not_null())
					.to_owned(),
			).await?;
		manager
			.create_table(
				Table::create()
					.table(Metrics::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Metrics::Id)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Metrics::Name).string().not_null())
					.col(ColumnDef::new(Metrics::Position).integer().not_null())
					.col(ColumnDef::new(Metrics::PanelId).big_integer().not_null())
					.col(ColumnDef::new(Metrics::SourceId).big_integer().not_null())
					.col(ColumnDef::new(Metrics::QueryX).string().not_null())
					.col(ColumnDef::new(Metrics::QueryY).string().not_null())
					.col(ColumnDef::new(Metrics::Color).integer().not_null())
					.to_owned(),
			).await?;
		manager
			.create_table(
				Table::create()
					.table(Points::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Points::Id)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Points::MetricId).big_integer().not_null())
					.col(ColumnDef::new(Points::X).double().not_null())
					.col(ColumnDef::new(Points::Y).double().not_null())
					.to_owned(),
			).await?;
		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Panels::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Sources::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Metrics::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Points::Table).to_owned())
			.await?;
		Ok(())
	}
}

#[derive(Iden)]
enum Panels {
	Table,
	Id,
	Name,
	Position,
	Timeserie,
	Height,
	Width,
	ViewScroll,
	LimitView,
	ViewSize,
	ReduceView,
	ViewChunks,
	ShiftView,
	ViewOffset,
	AverageView,
}

#[derive(Iden)]
enum Sources {
	Table,
	Id,
	Name,
	Position,
	Enabled,
	Url,
	Interval,
}

#[derive(Iden)]
enum Metrics {
	Table,
	Id,
	Name,
	Position,
	PanelId,
	SourceId,
	QueryX,
	QueryY,
	Color,
}

#[derive(Iden)]
enum Points {
	Table,
	Id,
	MetricId,
	X,
	Y,
}
