pub mod platform;
pub mod registry;
pub mod store;

pub use platform::{Architecture, OsType, PlatformDetector, PlatformInfo};
pub use registry::{InstalledToolsRegistry, InstalledVersion, ToolManager};
pub use store::StoreManager;
