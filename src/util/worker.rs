use tokio::{runtime::Runtime, sync::oneshot::Sender};

pub(crate) trait BackgroundWorker {
	fn start() -> Self;  // TODO make it return an error? Can we even do anything without a background worker
	fn task<T>(&self, task:T) where T : std::future::Future<Output = ()> + core::marker::Send + 'static;
	fn stop(self);   // TODO make it return an error? Can we even do anything without a background worker
}

pub(crate) struct NativeBackgroundWorker {
	runtime : Runtime,
	end_tx : Sender<bool>,
	worker : std::thread::JoinHandle<bool>,
}

impl BackgroundWorker for NativeBackgroundWorker {
	fn start() -> Self {
		let runtime = Runtime::new().expect("Failed creating Tokio runtime");
		let (end_tx, end_rx) = tokio::sync::oneshot::channel::<bool>();
		let r_handle = runtime.handle().clone();
		let worker = std::thread::spawn(move ||{
			r_handle.block_on(async {
				end_rx.await.expect("Error shutting down")
			})
		});
		NativeBackgroundWorker {
			runtime : runtime,
			end_tx : end_tx,
			worker : worker,
		}
	}

	fn task<T>(&self, task:T) where T : std::future::Future<Output = ()> + core::marker::Send + 'static {
		self.runtime.spawn(task);
	}

	fn stop(self) {
		self.end_tx.send(true).expect("Failed signaling termination");
		self.worker.join().expect("Failed joining main worker thread");
	}
}