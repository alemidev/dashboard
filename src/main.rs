mod gui;
mod data;
mod util;
mod worker;

use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing::info;
use tracing_subscriber::filter::filter_fn;

use clap::Parser;
use tokio::sync::watch;
use sea_orm::Database;

use worker::{surveyor_loop, visualizer_loop};
use gui::{
	// util::InternalLogger,
	App
};

/// Data gatherer and visualization tool
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
	/// Connection string for database to use
	db: String,

	/// Run background worker
	#[arg(long, default_value_t = false)]
	worker: bool,

	/// Run user interface
	#[arg(long, default_value_t = false)]
	gui: bool,

	/// Check interval for background worker
	#[arg(short, long, default_value_t = 5)]
	interval: u64,

	/// How often sources and metrics are refreshed
	#[arg(short, long, default_value_t = 300)]
	cache_time: u64,
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let args = CliArgs::parse();

	tracing_subscriber::registry()
		.with(LevelFilter::INFO)
		.with(filter_fn(|x| x.target() != "sqlx::query"))
		.with(tracing_subscriber::fmt::Layer::new())
		// .with(InternalLogger::new(store.clone()))
		.init();

	// // Set default file location
	// let mut store_path = dirs::data_dir().unwrap_or(PathBuf::from(".")); // TODO get cwd more consistently?
	// store_path.push("dashboard.db");
	//	let store =
	//		Arc::new(ApplicationState::new(store_path).expect("Failed creating application state"));
	

	let (panel_tx, panel_rx) = watch::channel(vec![]);
	let (source_tx, source_rx) = watch::channel(vec![]);
	let (metric_tx, metric_rx) = watch::channel(vec![]);
	let (point_tx, point_rx) = watch::channel(vec![]);
	let (view_tx, view_rx) = watch::channel(1440);
	
	let worker = std::thread::spawn(move || {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap()
			.block_on(async {
				let db = Database::connect(args.db.clone()).await.unwrap();
				info!(target: "launcher", "Connected to '{}'", args.db);

				let mut jobs = vec![];

				if args.worker {
					jobs.push(
						tokio::spawn(
							surveyor_loop(
								db.clone(),
								args.interval as i64,
								args.cache_time as i64,
							)
						)
					);
				}

				if args.gui {
					jobs.push(
						tokio::spawn(
							visualizer_loop(
								db.clone(),
								args.interval,
								args.cache_time as i64,
								panel_tx,
								source_tx,
								metric_tx,
								point_tx,
								view_rx,
							)
						)
					);
				}

				for job in jobs { job.await.unwrap() }

				info!(target: "launcher", "Stopping background worker");
			})
	});

	if args.gui {
		let native_options = eframe::NativeOptions::default();

		eframe::run_native(
			// TODO replace this with a loop that ends so we can cleanly exit the background worker
			"dashboard",
			native_options,
			Box::new(
				move |cc| Box::new(
					App::new(
						cc,
						panel_rx,
						source_rx,
						metric_rx,
						point_rx,
						view_tx,
					)
				)
			),
		);
	}

	worker.join().unwrap();
}
