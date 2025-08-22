// Dashboard Manager - Serves the MindLink management dashboard
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router,
    routing::get,
    response::{Html, IntoResponse},
    http::StatusCode,
};
use tower_http::services::ServeDir;
use tokio::net::TcpListener;
use std::path::PathBuf;
use std::net::SocketAddr;

// Handler function to serve the index.html file
async fn serve_index() -> impl IntoResponse {
    // Try multiple possible paths for the dist directory
    let possible_paths = ["../dist/index.html", "dist/index.html", "./dist/index.html"];
    
    for path in possible_paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            return Html(content).into_response();
        }
    }
    
    (StatusCode::NOT_FOUND, "Dashboard not found - dist/index.html not accessible").into_response()
}

#[derive(Debug)]
pub struct DashboardManager {
    port: u16,
    host: String,
    is_running: Arc<RwLock<bool>>,
}

impl DashboardManager {
    pub async fn new() -> Self {
        let available_port = Self::find_available_port("127.0.0.1", 3002).await
            .unwrap_or(3002); // Fallback to 3002 if detection fails
        
        println!("Using port {} for dashboard", available_port);
        
        Self {
            port: available_port,
            host: "127.0.0.1".to_string(),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    // Find the first available port starting from the given port
    async fn find_available_port(host: &str, start_port: u16) -> Option<u16> {
        for port in start_port..start_port + 100 { // Check up to 100 ports
            let addr: SocketAddr = format!("{}:{}", host, port).parse().ok()?;
            
            if TcpListener::bind(&addr).await.is_ok() {
                return Some(port);
            }
        }
        None
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if *self.is_running.read().await {
            return Ok(());
        }
        
        println!("Starting MindLink management dashboard...");
        
        // Create the web server to serve the dashboard
        let dist_dir = PathBuf::from("dist");
        
        let app = Router::new()
            .route("/", get(serve_index))
            .route("/dashboard", get(serve_index)) // Alternative route
            .nest_service("/assets", ServeDir::new(dist_dir.join("assets")))
            .fallback(serve_index); // Serve index.html for all other routes (SPA)
        
        let host = self.host.clone();
        let port = self.port;
        
        // Spawn the server in a background task
        let is_running = self.is_running.clone();
        tokio::spawn(async move {
            let listener = match TcpListener::bind(format!("{}:{}", host, port)).await {
                Ok(listener) => listener,
                Err(e) => {
                    eprintln!("Failed to bind dashboard to {}:{}: {}", host, port, e);
                    return;
                }
            };
            
            println!("MindLink dashboard started on {}:{}", host, port);
            *is_running.write().await = true;
            
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Dashboard server error: {}", e);
                *is_running.write().await = false;
            }
        });
        
        // Wait a moment for the server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        if !*self.is_running.read().await {
            return Ok(());
        }
        
        println!("Stopping MindLink dashboard...");
        *self.is_running.write().await = false;
        println!("MindLink dashboard stopped");
        
        Ok(())
    }
    
    pub async fn check_health(&self) -> Result<bool> {
        if !*self.is_running.read().await {
            return Ok(false);
        }
        
        // Make a health check request to dashboard
        let url = format!("http://{}:{}/", self.host, self.port);
        
        match reqwest::get(&url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    pub async fn get_local_url(&self) -> Option<String> {
        if *self.is_running.read().await {
            Some(format!("http://{}:{}", self.host, self.port))
        } else {
            None
        }
    }
    
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    pub async fn configure(&mut self, host: String, port: u16) {
        if *self.is_running.read().await {
            eprintln!("Cannot change configuration while dashboard is running");
            return;
        }
        
        self.host = host;
        self.port = port;
    }
    
    // Get dashboard status
    pub async fn get_status_info(&self) -> (bool, Option<String>) {
        let running = *self.is_running.read().await;
        let url = if running {
            Some(format!("http://{}:{}", self.host, self.port))
        } else {
            None
        };
        
        (running, url)
    }
}