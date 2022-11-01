mod gui;
mod data;
mod util;
mod worker;

use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing::{info, error};
use tracing_subscriber::filter::filter_fn;

use eframe::egui::Context;
use clap::Parser;
use tokio::sync::watch;
use sea_orm::Database;

use worker::visualizer::AppState;
use worker::surveyor_loop;
use util::{InternalLogger, InternalLoggerLayer};
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
	#[arg(short, long, default_value_t = 10)]
	interval: u64,

	/// How often sources and metrics are refreshed
	#[arg(short, long, default_value_t = 300)]
	cache_time: u64,

	/// How many log lines to keep in memory
	#[arg(short, long, default_value_t = 1000)]
	log_size: u64,
}

// When compiling for web:
#[cfg(target_arch = "wasm32")]
fn setup_tracing(_layer: InternalLoggerLayer) {
	// Make sure panics are logged using `console.error`.
	console_error_panic_hook::set_once();
	// Redirect tracing to console.log and friends:
	tracing_wasm::set_as_global_default();
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn setup_tracing(layer: InternalLoggerLayer) {
	tracing_subscriber::registry()
		.with(LevelFilter::INFO)
		.with(filter_fn(|x| x.target() != "sqlx::query"))
		.with(tracing_subscriber::fmt::Layer::new())
		.with(layer)
		.init();
}

fn main() {
	let args = CliArgs::parse();

	// TODO is there an alternative to this ugly botch?
	let (ctx_tx, ctx_rx) = watch::channel::<Option<Context>>(None);

	let (width_tx, width_rx) = watch::channel(0);
	let (run_tx, run_rx) = watch::channel(true);

	let logger = InternalLogger::new(args.log_size as usize);
	let logger_view = logger.view();

	setup_tracing(logger.layer());

	let state = AppState::new(
		width_rx,
		args.interval as i64,
		args.cache_time as i64,
	).unwrap();

	let view = state.view();
	let run_rx_clone = run_rx.clone();
	let db_uri = args.db.clone();
	
	let worker = std::thread::spawn(move || {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap()
			.block_on(async {
				let db = Database::connect(db_uri.clone()).await.unwrap();
				info!(target: "launcher", "Connected to '{}'", db_uri);

				let mut jobs = vec![];

				let run_rx_clone_clone = run_rx_clone.clone();

				jobs.push(
					tokio::spawn(async move {
						while *run_rx_clone_clone.borrow() {
							if let Some(ctx) = &*ctx_rx.borrow() {
								ctx.request_repaint();
							}
							tokio::time::sleep(std::time::Duration::from_secs(args.interval)).await;
						}
					})
				);

				jobs.push(
					tokio::spawn(logger.worker(run_rx_clone.clone()))
				);

				if args.worker {
					jobs.push(
						tokio::spawn(
							surveyor_loop(
								db.clone(),
								args.interval as i64,
								args.cache_time as i64,
								run_rx_clone.clone(),
							)
						)
					);
				}

				if args.gui {
					jobs.push(
						tokio::spawn(
							state.worker(db, run_rx_clone.clone())
						)
					);
				}

				for job in jobs { job.await.unwrap() }

				info!(target: "launcher", "Stopping background worker");
			})
	});

	if args.gui {
		let native_options = eframe::NativeOptions::default();

		info!(target: "launcher", "Starting native GUI");

		eframe::run_native(
			// TODO replace this with a loop that ends so we can cleanly exit the background worker
			"dashboard",
			native_options,
			Box::new(
				move |cc| {
					ctx_tx.send(Some(cc.egui_ctx.clone())).unwrap_or_else(|_| {
						error!(target: "launcher", "Could not share reference to egui context (won't be able to periodically refresh window)");
					});
					Box::new(
						App::new(
							cc,
							args.db,
							args.interval as i64,
							view,
							width_tx,
							logger_view,
						)
					)
				}
			),
		);

		info!(target: "launcher", "Stopping native GUI");

		run_tx.send(false).unwrap();
	}

	worker.join().unwrap();
}
