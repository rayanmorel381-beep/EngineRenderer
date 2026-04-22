/// Logs runtime messages only in debug builds.
#[macro_export]
macro_rules! runtime_log {
	($($arg:tt)*) => {
		if cfg!(debug_assertions) {
			::std::eprintln!($($arg)*);
		}
	};
}

/// Animation systems and timelines.
pub mod animation;
pub mod coremanager;
pub mod debug;
pub mod engine;
pub mod input;
pub mod scheduler;
pub mod simulation;
