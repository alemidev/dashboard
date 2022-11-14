mod gui;
mod data;
mod util;
mod worker;

use std::sync::Arc;

use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing::{info, error};
use tracing_subscriber::filter::filter_fn;

use eframe::egui::Context;
use clap::{Parser, Subcommand};
use tokio::sync::{watch, mpsc};
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
	/// Which mode to run in
	#[clap(subcommand)]
	mode: Mode,

	/// Check interval for background worker
	#[arg(short, long, default_value_t = 10)]
	interval: u64,

	/// How often sources and metrics are refreshed
	#[arg(short, long, default_value_t = 300)]
	cache_time: u64,

	/// How many log lines to keep in memory
	#[arg(long, default_value_t = 1000)]
	log_size: u64,


	#[arg(long)]
	log_file: Option<String>,
}

#[derive(Subcommand, Clone, Debug)]
enum Mode {
	/// Run as background service fetching sources from db
	Worker {
		/// Connection string for database to use
		#[arg(required = true)]
		db_uris: Vec<String>,
	},
	/// Run as foreground user interface displaying collected data
	GUI {
		/// Immediately connect to this database on startup
		#[arg(short, long)]
		db_uri: Option<String>,
	},
}

fn setup_tracing(layer: Option<InternalLoggerLayer>, log_to_file:Option<String>) {
	let file_layer = if let Some(path) = log_to_file {
		let file = std::fs::File::create(path).expect("Cannot open requested log file for writing");
		Some(tracing_subscriber::fmt::layer().with_ansi(false).with_writer(Arc::new(file)))
	} else {
		None
	};

	tracing_subscriber::registry()
		.with(LevelFilter::INFO)
		.with(filter_fn(|x| x.target() != "sqlx::query"))
		.with(tracing_subscriber::fmt::layer()) // stdout log
		.with(file_layer)
		.with(layer)
		.init();
}

fn main() {
	let args = CliArgs::parse();

	// TODO is there an alternative to this ugly botch?
	let (ctx_tx, ctx_rx) = watch::channel::<Option<Context>>(None);

	let (run_tx, run_rx) = watch::channel(true);

	match args.mode {
		Mode::Worker { db_uris } => {
			setup_tracing(None, args.log_file);

			let worker = std::thread::spawn(move || {
				tokio::runtime::Builder::new_multi_thread()
					.enable_all()
					.build()
					.unwrap()
					.block_on(async {
						let mut jobs = vec![];

						for (i, db_uri) in db_uris.iter().enumerate() {
							let db = match Database::connect(db_uri.clone()).await {
								Ok(v) => v,
								Err(e) => {
									error!(target: "worker", "Could not connect to db #{}: {:?}", i, e);
									return;
								}
							};

							info!(target: "worker", "Connected to #{}: '{}'", i, db_uri);

							jobs.push(
								tokio::spawn(
									surveyor_loop(
										db,
										args.interval as i64,
										args.cache_time as i64,
										run_rx.clone(),
										i,
									)
								)
							);
						}

						for (i, job) in jobs.into_iter().enumerate() {
							if let Err(e) = job.await {
								error!(target: "worker", "Could not join task #{}: {:?}", i, e);
							}
						}

						info!(target: "worker", "Stopping background worker");
					})
			});

			let (sigint_tx, sigint_rx) = std::sync::mpsc::channel(); // TODO can I avoid using a std channel?
			ctrlc::set_handler(move ||
				sigint_tx.send(()).expect("Could not send signal on channel")
			).expect("Could not set SIGINT handler");

			sigint_rx.recv().expect("Could not receive signal from channel");
			info!(target: "launcher", "Received SIGINT, stopping...");

			run_tx.send(false).unwrap_or(()); // ignore errors
			worker.join().expect("Failed joining worker thread");
		},

		Mode::GUI { db_uri } => {
			let (uri_tx, uri_rx) = mpsc::channel(10);
			let (width_tx, width_rx) = watch::channel(0);

			let logger = InternalLogger::new(args.log_size as usize);
			let logger_view = logger.view();

			setup_tracing(Some(logger.layer()), args.log_file);

			let state = match AppState::new(
				width_rx,
				uri_rx,
				args.interval as i64,
				args.cache_time as i64,
			) {
				Ok(s) => s,
				Err(e) => {
					error!(target: "launcher", "Could not create application state: {:?}", e);
					return;
				}
			};
			let view = state.view();
			
			let worker = std::thread::spawn(move || {
				tokio::runtime::Builder::new_current_thread()
					.enable_all()
					.build()
					.unwrap()
					.block_on(async {
						let mut jobs = vec![];

						let mut run_rx_clone_clone = run_rx.clone();

						jobs.push(
							tokio::spawn(async move {
								loop {
									// TODO probably state-worker can request a repaint directly, if we pass the
									// channel used to receive ctx
									tokio::select!{ // block on `run` too so that application can exit quickly
										_ = run_rx_clone_clone.changed() => {
											if ! *run_rx_clone_clone.borrow() { break; }
										},
										_ = tokio::time::sleep(std::time::Duration::from_secs(args.interval)) => {
											if let Some(ctx) = &*ctx_rx.borrow() {
												ctx.request_repaint();
											}
										},
									}
								}
							})
						);

						jobs.push(
							tokio::spawn(logger.worker(run_rx.clone()))
						);

						jobs.push(
							tokio::spawn(
								state.worker(run_rx.clone())
							)
						);

						for (i, job) in jobs.into_iter().enumerate() {
							if let Err(e) = job.await {
								error!(target: "worker", "Could not join task #{}: {:?}", i, e);
							}
						}

						info!(target: "worker", "Stopping background worker");
					})
			});

			let native_options = eframe::NativeOptions::default();

			info!(target: "launcher", "Starting native GUI");

			eframe::run_native(
				// TODO replace this with a loop that ends so we can cleanly exit the background worker
				"dashboard",
				native_options,
				Box::new(
					move |cc| {
						if let Err(_e) = ctx_tx.send(Some(cc.egui_ctx.clone())) {
							error!(target: "launcher", "Could not share reference to egui context (won't be able to periodically refresh window)");
						};
						Box::new(
							App::new(
								cc,
								db_uri,
								uri_tx,
								args.interval as i64,
								view,
								width_tx,
								logger_view,
							)
						)
					}
				),
			);

			info!(target: "launcher", "GUI quit, stopping background worker...");

			run_tx.send(false).unwrap_or(()); // ignore errors

			worker.join().expect("Failed joining worker thread");
		}
	}

}
