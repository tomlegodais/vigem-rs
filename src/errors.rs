use thiserror::Error;

#[derive(Debug, Error)]
pub enum VigemError {
    #[error("ViGEmBus device driver not found")]
    DeviceNotFound,
    #[error("Failed to allocate memory")]
    AllocationError,
}