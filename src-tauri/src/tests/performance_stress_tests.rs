//! Performance and Stress Tests
//!
//! This module contains performance benchmarks and stress tests to validate
//! that the MindLink application performs well under load and handles
//! resource constraints gracefully.

#[cfg(test)]
mod performance_stress_tests {
    use crate::managers::auth_manager::AuthManager;
    use crate::managers::config_manager::ConfigManager;
    use crate::managers::server_manager::ServerManager;
    use crate::managers::tunnel_manager::{TunnelManager, TunnelType};
    use futures::future::join_all;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::{Duration, Instant};

    /// Performance benchmark helper
    struct PerformanceBenchmark {
        start_time: Instant,
        operation_name: String,
    }

    impl PerformanceBenchmark {
        fn start(operation_name: &str) -> Self {
            println!("⏱️  Starting benchmark: {}", operation_name);
            Self {
                start_time: Instant::now(),
                operation_name: operation_name.to_string(),
            }
        }

        fn end(self) -> Duration {
            let duration = self.start_time.elapsed();
            println!("⏱️  Completed {}: {:?}", self.operation_name, duration);
            duration
        }

        fn end_with_ops(self, operations: usize) -> (Duration, f64) {
            let duration = self.start_time.elapsed();
            let ops_per_sec = operations as f64 / duration.as_secs_f64();
            println!(
                "⏱️  Completed {}: {:?} ({:.2} ops/sec)",
                self.operation_name, duration, ops_per_sec
            );
            (duration, ops_per_sec)
        }
    }

    #[tokio::test]
    async fn test_manager_creation_performance() {
        println!("🏃 Performance Test: Manager creation speed");

        // Benchmark AuthManager creation
        let bench = PerformanceBenchmark::start("AuthManager creation");
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");
        let auth_duration = bench.end();
        assert!(
            auth_duration < Duration::from_millis(1000),
            "AuthManager creation should be fast"
        );

        // Benchmark ServerManager creation
        let bench = PerformanceBenchmark::start("ServerManager creation");
        let _server_manager = ServerManager::new().await;
        let server_duration = bench.end();
        assert!(
            server_duration < Duration::from_millis(500),
            "ServerManager creation should be fast"
        );

        // Benchmark TunnelManager creation
        let bench = PerformanceBenchmark::start("TunnelManager creation");
        let _tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");
        let tunnel_duration = bench.end();
        assert!(
            tunnel_duration < Duration::from_millis(500),
            "TunnelManager creation should be fast"
        );

        // Benchmark ConfigManager creation
        let bench = PerformanceBenchmark::start("ConfigManager creation");
        let _config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config_duration = bench.end();
        assert!(
            config_duration < Duration::from_millis(1000),
            "ConfigManager creation should be fast"
        );

        println!("✅ Manager creation performance test successful");
    }

    #[tokio::test]
    async fn test_concurrent_manager_creation() {
        println!("🏃 Stress Test: Concurrent manager creation");

        const NUM_CONCURRENT: usize = 10;
        let bench = PerformanceBenchmark::start("Concurrent manager creation");

        // Create managers concurrently
        let mut handles = vec![];

        for i in 0..NUM_CONCURRENT {
            let handle = tokio::spawn(async move {
                let auth_result = AuthManager::new().await;
                let server_manager = ServerManager::new().await;
                let tunnel_result = TunnelManager::new().await;
                let config_result = ConfigManager::new().await;

                println!(
                    "   Concurrent creation {}: auth={}, tunnel={}, config={}",
                    i,
                    auth_result.is_ok(),
                    tunnel_result.is_ok(),
                    config_result.is_ok()
                );

                // Return success count
                let success_count = [
                    auth_result.is_ok(),
                    true, // ServerManager::new() doesn't return Result
                    tunnel_result.is_ok(),
                    config_result.is_ok(),
                ]
                .iter()
                .filter(|&&x| x)
                .count();

                success_count
            });
            handles.push(handle);
        }

        let results = join_all(handles).await;
        let (duration, ops_per_sec) = bench.end_with_ops(NUM_CONCURRENT);

        // Verify all concurrent operations completed successfully
        let total_successful = results
            .iter()
            .map(|r| r.as_ref().map(|res| *res).unwrap_or(0))
            .sum::<usize>();

        println!(
            "   Total successful operations: {}/{}",
            total_successful,
            NUM_CONCURRENT * 4
        );
        assert!(
            total_successful >= NUM_CONCURRENT * 3,
            "At least 75% of operations should succeed"
        );
        assert!(
            ops_per_sec >= 5.0,
            "Should handle at least 5 creation operations per second"
        );

        println!("✅ Concurrent manager creation test successful");
    }

    #[tokio::test]
    async fn test_rapid_state_queries() {
        println!("🏃 Performance Test: Rapid state queries");

        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");
        let server_manager = ServerManager::new().await;
        let tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        const NUM_QUERIES: usize = 1000;

        // Benchmark auth status queries
        let bench = PerformanceBenchmark::start("Auth status queries");
        for _ in 0..NUM_QUERIES {
            let _ = auth_manager.is_authenticated().await;
        }
        let (auth_duration, auth_ops) = bench.end_with_ops(NUM_QUERIES);
        assert!(
            auth_ops >= 500.0,
            "Auth queries should be fast (>500 ops/sec)"
        );

        // Benchmark server status queries
        let bench = PerformanceBenchmark::start("Server status queries");
        for _ in 0..NUM_QUERIES {
            let _ = server_manager.is_running().await;
        }
        let (server_duration, server_ops) = bench.end_with_ops(NUM_QUERIES);
        assert!(
            server_ops >= 500.0,
            "Server queries should be fast (>500 ops/sec)"
        );

        // Benchmark tunnel status queries
        let bench = PerformanceBenchmark::start("Tunnel status queries");
        for _ in 0..NUM_QUERIES {
            let _ = tunnel_manager.is_connected().await;
        }
        let (tunnel_duration, tunnel_ops) = bench.end_with_ops(NUM_QUERIES);
        assert!(
            tunnel_ops >= 500.0,
            "Tunnel queries should be fast (>500 ops/sec)"
        );

        println!("✅ Rapid state queries test successful");
    }

    #[tokio::test]
    async fn test_memory_usage_under_load() {
        println!("🏃 Stress Test: Memory usage under load");

        const NUM_MANAGERS: usize = 50;
        let bench = PerformanceBenchmark::start("Memory stress test");

        // Create multiple managers to test memory usage
        let mut auth_managers = Vec::new();
        let mut server_managers = Vec::new();
        let mut tunnel_managers = Vec::new();
        let mut config_managers = Vec::new();

        for i in 0..NUM_MANAGERS {
            if let Ok(auth) = AuthManager::new().await {
                auth_managers.push(auth);
            }

            server_managers.push(ServerManager::new().await);

            if let Ok(tunnel) = TunnelManager::new().await {
                tunnel_managers.push(tunnel);
            }

            if let Ok(config) = ConfigManager::new().await {
                config_managers.push(config);
            }

            // Log progress every 10 managers
            if (i + 1) % 10 == 0 {
                println!("   Created {} managers", i + 1);
            }
        }

        // Perform operations on all managers to test memory stability
        for (i, auth) in auth_managers.iter().enumerate() {
            let _ = auth.is_authenticated().await;
            if i % 10 == 0 {
                println!("   Tested auth manager {}", i);
            }
        }

        for (i, server) in server_managers.iter().enumerate() {
            let _ = server.is_running().await;
            if i % 10 == 0 {
                println!("   Tested server manager {}", i);
            }
        }

        let duration = bench.end();

        // Clean up explicitly to test deallocation
        drop(auth_managers);
        drop(server_managers);
        drop(tunnel_managers);
        drop(config_managers);

        println!(
            "   Created and tested {} managers in {:?}",
            NUM_MANAGERS, duration
        );
        assert!(
            duration < Duration::from_secs(30),
            "Memory stress test should complete in reasonable time"
        );

        println!("✅ Memory usage under load test successful");
    }

    #[tokio::test]
    async fn test_concurrent_operations_performance() {
        println!("🏃 Performance Test: Concurrent operations performance");

        let auth_manager = Arc::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        );
        let server_manager = Arc::new(tokio::sync::RwLock::new(ServerManager::new().await));
        let tunnel_manager = Arc::new(
            TunnelManager::new()
                .await
                .expect("Failed to create tunnel manager"),
        );
        let config_manager = Arc::new(
            ConfigManager::new()
                .await
                .expect("Failed to create config manager"),
        );

        const NUM_CONCURRENT_OPS: usize = 100;
        let bench = PerformanceBenchmark::start("Concurrent operations");

        let mut handles = vec![];

        // Launch concurrent operations
        for i in 0..NUM_CONCURRENT_OPS {
            let auth_clone = auth_manager.clone();
            let server_clone = server_manager.clone();
            let tunnel_clone = tunnel_manager.clone();
            let config_clone = config_manager.clone();

            let handle = tokio::spawn(async move {
                match i % 4 {
                    0 => {
                        // Auth operations
                        let _ = auth_clone.is_authenticated().await;
                        let _ = auth_clone.get_access_token();
                    },
                    1 => {
                        // Server operations
                        let server = server_clone.read().await;
                        let _ = server.is_running().await;
                        let _ = server.get_local_url().await;
                    },
                    2 => {
                        // Tunnel operations
                        let _ = tunnel_clone.is_connected().await;
                        let _ = tunnel_clone.get_current_url().await;
                    },
                    3 => {
                        // Config operations
                        let _ = config_clone.get_config().await;
                        let _ = config_clone.get_config().await; // Test double access
                    },
                    _ => unreachable!(),
                }
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results = join_all(handles).await;
        let (duration, ops_per_sec) = bench.end_with_ops(NUM_CONCURRENT_OPS);

        // Verify all operations completed successfully
        let successful_ops = results.iter().filter(|r| r.is_ok()).count();
        println!(
            "   Successful operations: {}/{}",
            successful_ops, NUM_CONCURRENT_OPS
        );

        assert_eq!(
            successful_ops, NUM_CONCURRENT_OPS,
            "All concurrent operations should succeed"
        );
        assert!(
            ops_per_sec >= 50.0,
            "Should handle at least 50 concurrent operations per second"
        );

        println!("✅ Concurrent operations performance test successful");
    }

    #[tokio::test]
    async fn test_configuration_update_performance() {
        println!("🏃 Performance Test: Configuration update performance");

        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        const NUM_UPDATES: usize = 100;
        let bench = PerformanceBenchmark::start("Configuration updates");

        // Perform rapid config updates
        for i in 0..NUM_UPDATES {
            let mut config = config_manager.get_config().await;
            config.server.port = 3000 + (i as u16);
            config.server.host = format!("127.0.0.{}", i % 255);

            let update_result = config_manager.update_config(config).await;
            assert!(update_result.is_ok(), "Config update {} should succeed", i);

            if i % 20 == 0 {
                println!("   Completed {} config updates", i);
            }
        }

        let (duration, ops_per_sec) = bench.end_with_ops(NUM_UPDATES);

        // Verify final config state
        let final_config = config_manager.get_config().await;
        assert_eq!(final_config.server.port, 3000 + (NUM_UPDATES as u16) - 1);

        assert!(
            ops_per_sec >= 10.0,
            "Should handle at least 10 config updates per second"
        );
        assert!(
            duration < Duration::from_secs(15),
            "Config updates should complete in reasonable time"
        );

        println!("✅ Configuration update performance test successful");
    }

    #[tokio::test]
    async fn test_error_handling_performance() {
        println!("🏃 Performance Test: Error handling performance");

        let mut auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");
        let mut server_manager = ServerManager::new().await;
        let mut tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        const NUM_ERROR_OPS: usize = 50;
        let bench = PerformanceBenchmark::start("Error handling operations");

        // Generate various error conditions and measure handling performance
        let mut error_count = 0;

        for i in 0..NUM_ERROR_OPS {
            match i % 3 {
                0 => {
                    // Auth errors
                    let result = auth_manager.refresh_tokens().await;
                    if result.is_err() {
                        error_count += 1;
                    }
                },
                1 => {
                    // Server errors
                    let auth_shared = Arc::new(RwLock::new(
                        AuthManager::new()
                            .await
                            .expect("Failed to create auth manager"),
                    ));
                    let result = server_manager.start(auth_shared).await;
                    if result.is_err() {
                        error_count += 1;
                    }
                    // Try to stop (might also error)
                    let _ = server_manager.stop().await;
                },
                2 => {
                    // Tunnel errors
                    let result = tunnel_manager.create_tunnel().await;
                    if result.is_err() {
                        error_count += 1;
                    }
                    // Try to close (might also error)
                    let _ = tunnel_manager.close_tunnel().await;
                },
                _ => unreachable!(),
            }
        }

        let (duration, ops_per_sec) = bench.end_with_ops(NUM_ERROR_OPS);

        println!(
            "   Generated {} errors out of {} operations",
            error_count, NUM_ERROR_OPS
        );
        assert!(
            error_count > 0,
            "Should have generated some errors for testing"
        );
        assert!(
            ops_per_sec >= 5.0,
            "Error handling should be reasonably fast (>5 ops/sec)"
        );
        assert!(
            duration < Duration::from_secs(20),
            "Error handling should complete in reasonable time"
        );

        println!("✅ Error handling performance test successful");
    }

    #[tokio::test]
    async fn test_health_check_performance() {
        println!("🏃 Performance Test: Health check performance");

        let server_manager = ServerManager::new().await;
        let tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        const NUM_HEALTH_CHECKS: usize = 200;

        // Benchmark server health checks
        let bench = PerformanceBenchmark::start("Server health checks");
        for _ in 0..NUM_HEALTH_CHECKS {
            let _ = server_manager.check_health().await;
        }
        let (server_duration, server_ops) = bench.end_with_ops(NUM_HEALTH_CHECKS);
        assert!(
            server_ops >= 20.0,
            "Server health checks should be reasonably fast (>20 ops/sec)"
        );

        // Benchmark tunnel health checks
        let bench = PerformanceBenchmark::start("Tunnel health checks");
        for _ in 0..NUM_HEALTH_CHECKS {
            let _ = tunnel_manager.check_health().await;
        }
        let (tunnel_duration, tunnel_ops) = bench.end_with_ops(NUM_HEALTH_CHECKS);
        assert!(
            tunnel_ops >= 20.0,
            "Tunnel health checks should be reasonably fast (>20 ops/sec)"
        );

        println!("✅ Health check performance test successful");
    }

    #[tokio::test]
    async fn test_resource_contention_handling() {
        println!("🏃 Stress Test: Resource contention handling");

        let config_manager = Arc::new(
            ConfigManager::new()
                .await
                .expect("Failed to create config manager"),
        );

        const NUM_CONTENDING_TASKS: usize = 20;
        let bench = PerformanceBenchmark::start("Resource contention handling");

        // Create contending tasks that access shared resources
        let mut handles = vec![];

        for i in 0..NUM_CONTENDING_TASKS {
            let config_clone = config_manager.clone();
            let handle = tokio::spawn(async move {
                // Mix of read and write operations to test contention
                for j in 0..10 {
                    if j % 3 == 0 {
                        // Write operation
                        let mut config = config_clone.get_config().await;
                        config.server.port = 3000 + (i as u16 * 10) + j;
                        let _ = config_clone.update_config(config).await;
                    } else {
                        // Read operation
                        let _ = config_clone.get_config().await;
                        let _ = config_clone.get_config().await; // Test double read
                    }
                }
                println!("   Contending task {} completed", i);
            });
            handles.push(handle);
        }

        // Wait for all contending tasks to complete
        let results = join_all(handles).await;
        let duration = bench.end();

        let successful_tasks = results.iter().filter(|r| r.is_ok()).count();
        println!(
            "   Successful tasks: {}/{}",
            successful_tasks, NUM_CONTENDING_TASKS
        );

        assert_eq!(
            successful_tasks, NUM_CONTENDING_TASKS,
            "All contending tasks should complete successfully"
        );
        assert!(
            duration < Duration::from_secs(30),
            "Resource contention should resolve in reasonable time"
        );

        println!("✅ Resource contention handling test successful");
    }
}
