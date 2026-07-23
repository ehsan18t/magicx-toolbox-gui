pub mod elevation;
pub mod firewall_service;
pub mod hosts_service;
pub mod registry_service;
pub mod registry_value;
pub mod scheduler_service;
pub mod service_control;
pub mod system_info_service;

// Re-export trusted_installer for backwards compatibility
pub use elevation as trusted_installer;
