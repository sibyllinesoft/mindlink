//! # Manager Modules
//!
//! This module contains the core business logic managers for the MindLink application.
//! These managers handle different aspects of the application's functionality, from
//! authentication and server management to tunneling and configuration.
//!
//! ## Architecture
//!
//! The manager system follows a modular architecture where each manager is responsible
//! for a specific domain of functionality:
//!
//! - **Authentication**: OAuth2 flows and token management
//! - **Server**: HTTP API server lifecycle and health monitoring  
//! - **Tunnel**: Cloudflare tunnel creation and management
//! - **Configuration**: Application settings and persistence
//! - **Binary**: External binary management and execution
//! - **Bifrost**: Dashboard and monitoring interface
//! - **Dashboard**: Web interface for system management
//!
//! ## Usage Pattern
//!
//! Managers are typically initialized during application startup and accessed
//! through the global `AppState`. Each manager provides both synchronous and
//! asynchronous methods depending on the operation complexity.
//!
//! ```rust,no_run
//! use crate::managers::auth_manager::AuthManager;
//! use crate::managers::server_manager::ServerManager;
//!
//! // Managers are typically created during app initialization
//! let auth_manager = AuthManager::new().await?;
//! let server_manager = ServerManager::new(/* config */).await?;
//! ```
//!
//! ## Error Handling
//!
//! All managers use the unified [`crate::error::MindLinkError`] type for error handling,
//! providing structured error information with context for debugging and
//! user-friendly error messages.
//!
//! ## Thread Safety
//!
//! All managers are designed to be thread-safe and can be safely shared
//! between multiple async tasks using `Arc<Manager>` patterns.

pub mod auth_manager;
pub mod bifrost_manager;
pub mod binary_manager;
pub mod config_manager;
pub mod dashboard_manager;
pub mod server_manager;
pub mod tunnel_manager;
