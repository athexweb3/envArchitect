use crate::entities::tool::Tool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Package not found: {0}")]
    NotFound(String),
    #[error("Installation failed: {0}")]
    InstallFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

pub trait PackageManager {
    fn install(&self, tool: &Tool) -> Result<(), PackageError>;
    fn is_installed(&self, tool: &Tool) -> Result<bool, PackageError>;
}
