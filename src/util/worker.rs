use tokio::{runtime::{Handle, Runtime, Builder}, sync::oneshot::Sender, time::{sleep, Duration}};

pub(crate) trait BackgroundWorker {
	fn start() -> Self;  // TODO make it return an error? Can we even do anything without a background worker
	fn task<T>(&self, task:T) where T : std::future::Future<Output = ()> + core::marker::Send + 'static;
	fn stop(self);   // TODO make it return an error? Can we even do anything without a background worker
}

pub(crate) struct NativeBackgroundWorker {
	runtime : Handle,
	// end_tx : Sender<bool>,
	worker : std::thread::JoinHandle<bool>,
}

impl BackgroundWorker for NativeBackgroundWorker {
	fn start() -> Self {
		let (rt_tx, rt_rx) = tokio::sync::oneshot::channel::<Handle>();
		let worker = std::thread::spawn(|| {
			let runtime = Builder::new_multi_thread()
				.worker_threads(1)
				.enable_all()
				.build()
				.unwrap();
			rt_tx.send(runtime.handle().clone()).unwrap();
			runtime.block_on(async {
				loop {
		 			println!("keepalive loop");
		 			sleep(Duration::from_secs(1)).await;
				}
			})
		});
		NativeBackgroundWorker {
			runtime : rt_rx.blocking_recv().unwrap(),
			// end_tx : end_tx,
			worker : worker,
		}
	}

	fn task<T>(&self, task:T) where T : std::future::Future<Output = ()> + core::marker::Send + 'static {
		self.runtime.spawn(task);
	}

	fn stop(self) {
		// self.end_tx.send(true).expect("Failed signaling termination");
		// self.worker.join().expect("Failed joining main worker thread");
	}
}