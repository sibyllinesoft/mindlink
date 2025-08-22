// Manager modules - Rust implementations of the JavaScript managers

pub mod auth_manager;
pub mod server_manager;
pub mod tunnel_manager;
pub mod config_manager;
pub mod bifrost_manager;
pub mod dashboard_manager;
pub mod binary_manager;

// Re-export for convenience (allow unused as these are public API)
#[allow(unused_imports)]
pub use auth_manager::AuthManager;
#[allow(unused_imports)]
pub use server_manager::ServerManager;
#[allow(unused_imports)]
pub use tunnel_manager::TunnelManager;
#[allow(unused_imports)]
pub use config_manager::ConfigManager;
#[allow(unused_imports)]
pub use bifrost_manager::BifrostManager;
#[allow(unused_imports)]
pub use dashboard_manager::DashboardManager;
#[allow(unused_imports)]
pub use binary_manager::BinaryManager;