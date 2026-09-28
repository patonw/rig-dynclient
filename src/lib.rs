pub mod builder;
pub mod completion;

pub use builder::DynClientBuilder;

// Re-exports for convenience
#[cfg(feature = "rmcp")]
pub use rmcp;

pub use rig;
pub use rig_agent;
pub use rig_core;
