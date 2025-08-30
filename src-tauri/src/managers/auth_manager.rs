// Authentication Manager - OAuth2 PKCE Implementation with enterprise-grade error handling
use anyhow::{anyhow, Result};
use axum::{extract::Query, response::Html, routing::get, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use url::Url;

use crate::error::{MindLinkError, MindLinkResult};
use crate::{auth_error, log_error, log_info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthCallbackQuery {
    code: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
    state: Option<String>,
}

struct OAuthState {
    #[allow(dead_code)]
    code_verifier: String,
    #[allow(dead_code)]
    state: String,
    auth_result: Arc<RwLock<Option<MindLinkResult<String>>>>,
}

// Allow dead_code for OAuthState fields that might not be used in all tests
#[allow(dead_code)]
impl OAuthState {
    fn new(code_verifier: String, state: String) -> Self {
        Self {
            code_verifier,
            state,
            auth_result: Arc::new(RwLock::new(None)),
        }
    }
}

// OpenAI OAuth configuration
const OPENAI_AUTH_URL: &str = "https://auth0.openai.com/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth0.openai.com/oauth/token";
const CLIENT_ID: &str = "TdJIcbe16WoTHtN95nyywh5E4yOo6ItG"; // OpenAI's public client ID
const SCOPE: &str =
    "openid profile email offline_access model.request model.read organization.read";

#[derive(Debug)]
pub struct AuthManager {
    auth_path: PathBuf,
    tokens: Option<AuthTokens>,
}

impl AuthManager {
    /// Create a new AuthManager with proper error handling and token validation
    pub async fn new() -> MindLinkResult<Self> {
        let auth_dir = dirs::home_dir()
            .ok_or_else(|| MindLinkError::SystemResource {
                message: "Cannot determine home directory".to_string(),
                resource_type: "home directory".to_string(),
                source: None,
            })?
            .join(".mindlink");

        let auth_path = auth_dir.join("auth.json");

        // Ensure directory exists
        fs::create_dir_all(&auth_dir)
            .await
            .map_err(|e| MindLinkError::FileSystem {
                message: "Failed to create auth directory".to_string(),
                path: Some(auth_dir.to_string_lossy().to_string()),
                operation: "create directory".to_string(),
                source: Some(e.into()),
            })?;

        log_info!("AuthManager", "Initializing authentication system");

        let mut manager = Self {
            auth_path,
            tokens: None,
        };

        // Load and validate existing tokens
        match manager.load_tokens().await {
            Ok(_) => {
                log_info!("AuthManager", "Existing tokens loaded successfully");

                // Validate tokens on startup
                if let Err(validation_err) = manager.validate_tokens_on_startup().await {
                    log_error!("AuthManager", validation_err);
                    manager.tokens = None; // Clear invalid tokens
                }
            },
            Err(_e) => {
                log_info!(
                    "AuthManager",
                    "No existing auth tokens found, will require authentication"
                );
                // Not an error - just means user needs to authenticate
            },
        }

        log_info!("AuthManager", "Authentication system initialized");

        Ok(manager)
    }

    /// Validate tokens on startup and attempt silent refresh if needed
    async fn validate_tokens_on_startup(&mut self) -> MindLinkResult<()> {
        if let Some(tokens) = &self.tokens {
            let now = Utc::now();
            let expires_soon = now + Duration::minutes(5);

            if tokens.expires_at <= expires_soon {
                log_info!(
                    "AuthManager",
                    "Tokens expiring soon, attempting silent refresh"
                );

                match self.refresh_tokens_silently().await {
                    Ok(_) => {
                        log_info!("AuthManager", "Tokens refreshed successfully on startup");
                        Ok(())
                    },
                    Err(refresh_err) => {
                        log_error!("AuthManager", refresh_err.clone());
                        Err(refresh_err)
                    },
                }
            } else {
                log_info!("AuthManager", "Tokens are valid and not expiring soon");
                Ok(())
            }
        } else {
            Err(auth_error!("No tokens available for validation"))
        }
    }

    /// Silently refresh tokens using the refresh token
    async fn refresh_tokens_silently(&mut self) -> MindLinkResult<()> {
        let current_tokens = self
            .tokens
            .as_ref()
            .ok_or_else(|| auth_error!("No tokens available for refresh"))?;

        if current_tokens.refresh_token.is_empty() {
            return Err(auth_error!("No refresh token available"));
        }

        log_info!("AuthManager", "Attempting silent token refresh");

        let client = reqwest::Client::new();

        let mut refresh_params = HashMap::new();
        refresh_params.insert("grant_type", "refresh_token");
        refresh_params.insert("refresh_token", &current_tokens.refresh_token);
        refresh_params.insert("client_id", CLIENT_ID);

        let response = client
            .post(OPENAI_TOKEN_URL)
            .form(&refresh_params)
            .send()
            .await
            .map_err(|e| auth_error!("Failed to send refresh token request", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(auth_error!(format!("Token refresh failed: {}", error_text)));
        }

        let refresh_response: RefreshTokenResponse = response
            .json()
            .await
            .map_err(|e| auth_error!("Failed to parse refresh token response", e))?;

        // Update tokens with refreshed values
        let new_tokens = AuthTokens {
            access_token: refresh_response.access_token,
            refresh_token: refresh_response
                .refresh_token
                .unwrap_or(current_tokens.refresh_token.clone()),
            expires_at: Utc::now()
                + Duration::seconds(refresh_response.expires_in.unwrap_or(3600) as i64),
            token_type: refresh_response.token_type,
        };

        self.tokens = Some(new_tokens);
        self.save_tokens().await?;

        log_info!("AuthManager", "Tokens refreshed and saved successfully");

        Ok(())
    }

    pub async fn is_authenticated(&self) -> bool {
        if let Some(tokens) = &self.tokens {
            // Check if tokens are still valid (with 5 minute buffer)
            let buffer_time = Utc::now() + Duration::minutes(5);
            tokens.expires_at > buffer_time
        } else {
            false
        }
    }

    pub async fn login(&mut self) -> Result<()> {
        println!("🔐 Starting OAuth2 PKCE authentication flow...");

        // Generate PKCE parameters
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier)?;
        let state = Self::generate_state();

        // Find an available port for the callback server
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let callback_port = listener.local_addr()?.port();
        let redirect_uri = format!("http://127.0.0.1:{}/callback", callback_port);

        println!(
            "📡 Starting local callback server on port {}",
            callback_port
        );

        // Prepare OAuth state
        let oauth_state = Arc::new(OAuthState {
            code_verifier: code_verifier.clone(),
            state: state.clone(),
            auth_result: Arc::new(RwLock::new(None)),
        });

        // Build authorization URL
        let auth_url = Self::build_auth_url(&redirect_uri, &code_challenge, &state)?;
        println!("🌐 Opening browser for authentication...");

        // Open browser using system command
        if let Err(e) = Self::open_browser(&auth_url).await {
            println!(
                "⚠️ Failed to open browser automatically: {}. Please open this URL manually:",
                e
            );
            println!("    {}", auth_url);
        }

        // Start callback server and wait for response
        let auth_code = self.handle_callback_server(listener, oauth_state).await?;

        // Exchange authorization code for tokens
        let tokens = self
            .exchange_code_for_tokens(&auth_code, &code_verifier, &redirect_uri)
            .await?;

        // Store tokens
        self.tokens = Some(tokens);
        self.save_tokens().await?;

        println!("✅ Authentication successful!");
        Ok(())
    }

    pub async fn refresh_tokens(&mut self) -> Result<()> {
        let tokens = self
            .tokens
            .as_ref()
            .ok_or_else(|| anyhow!("No tokens available to refresh"))?;

        println!("🔄 Refreshing authentication tokens...");

        let client = reqwest::Client::new();
        let mut form_params = HashMap::new();
        form_params.insert("grant_type", "refresh_token");
        form_params.insert("refresh_token", &tokens.refresh_token);
        form_params.insert("client_id", CLIENT_ID);

        let response = client
            .post(OPENAI_TOKEN_URL)
            .form(&form_params)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Token refresh failed: {} - {}", status, error_text));
        }

        let refresh_response: RefreshTokenResponse = response.json().await?;

        let expires_at = if let Some(expires_in) = refresh_response.expires_in {
            Utc::now() + Duration::seconds(expires_in as i64)
        } else {
            Utc::now() + Duration::hours(1) // Default 1 hour if not specified
        };

        let new_tokens = AuthTokens {
            access_token: refresh_response.access_token,
            refresh_token: refresh_response
                .refresh_token
                .unwrap_or(tokens.refresh_token.clone()),
            expires_at,
            token_type: refresh_response.token_type,
        };

        self.tokens = Some(new_tokens);
        self.save_tokens().await?;

        println!("✅ Tokens refreshed successfully!");
        Ok(())
    }

    pub async fn logout(&mut self) -> Result<()> {
        self.tokens = None;

        // Remove auth file
        if self.auth_path.exists() {
            fs::remove_file(&self.auth_path).await?;
        }

        println!("Logged out successfully");
        Ok(())
    }

    pub fn get_access_token(&self) -> Option<&str> {
        self.tokens.as_ref().map(|t| t.access_token.as_str())
    }

    async fn load_tokens(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.auth_path).await?;

        // First try to deserialize with the new format (with token_type field)
        match serde_json::from_str::<AuthTokens>(&content) {
            Ok(tokens) => {
                self.tokens = Some(tokens);
                Ok(())
            },
            Err(_) => {
                // Try to deserialize with the old format (without token_type field) for backward compatibility
                #[derive(Debug, Clone, Serialize, Deserialize)]
                struct OldAuthTokens {
                    pub access_token: String,
                    pub refresh_token: String,
                    pub expires_at: DateTime<Utc>,
                }

                match serde_json::from_str::<OldAuthTokens>(&content) {
                    Ok(old_tokens) => {
                        // Migrate to new format
                        let new_tokens = AuthTokens {
                            access_token: old_tokens.access_token,
                            refresh_token: old_tokens.refresh_token,
                            expires_at: old_tokens.expires_at,
                            token_type: "Bearer".to_string(), // Default for OpenAI
                        };
                        self.tokens = Some(new_tokens);
                        // Save in new format
                        self.save_tokens().await?;
                        Ok(())
                    },
                    Err(e) => Err(anyhow!("Failed to parse token file: {}", e)),
                }
            },
        }
    }

    async fn save_tokens(&self) -> Result<()> {
        if let Some(tokens) = &self.tokens {
            let json = serde_json::to_string_pretty(tokens)?;
            fs::write(&self.auth_path, json).await?;
        }
        Ok(())
    }

    pub async fn ensure_valid_tokens(&mut self) -> Result<()> {
        if !self.is_authenticated().await {
            if self.tokens.is_some() {
                // Try to refresh first
                if let Err(e) = self.refresh_tokens().await {
                    println!("⚠️ Token refresh failed: {}", e);
                    // If refresh fails, need to login again
                    self.login().await?;
                }
            } else {
                // No tokens, need to login
                self.login().await?;
            }
        }
        Ok(())
    }

    // PKCE helper methods
    fn generate_code_verifier() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    fn generate_code_challenge(code_verifier: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let digest = hasher.finalize();
        Ok(URL_SAFE_NO_PAD.encode(&digest))
    }

    fn generate_state() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    fn build_auth_url(redirect_uri: &str, code_challenge: &str, state: &str) -> Result<String> {
        let mut url = Url::parse(OPENAI_AUTH_URL)?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", CLIENT_ID)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", SCOPE)
            .append_pair("state", state)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("prompt", "login")
            .append_pair("audience", "https://api.openai.com/v1");
        Ok(url.to_string())
    }

    async fn open_browser(url: &str) -> Result<()> {
        // Use system commands to open browser
        #[cfg(target_os = "linux")]
        {
            tokio::process::Command::new("xdg-open").arg(url).spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            tokio::process::Command::new("open").arg(url).spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            tokio::process::Command::new("cmd")
                .args(["/C", "start", url])
                .spawn()?;
        }

        Ok(())
    }

    async fn handle_callback_server(
        &self,
        listener: TcpListener,
        oauth_state: Arc<OAuthState>,
    ) -> Result<String> {
        println!("⏳ Waiting for authentication callback...");

        // Create the callback router
        let app = Router::new().route(
            "/callback",
            get({
                let oauth_state = oauth_state.clone();
                move |query: Query<AuthCallbackQuery>| Self::handle_callback(query, oauth_state)
            }),
        );

        // Use axum's serve function with our listener
        let server = axum::serve(listener, app);

        // Set a timeout for the authentication process
        let timeout_duration = std::time::Duration::from_secs(300); // 5 minutes

        let oauth_state_clone = oauth_state.clone();

        // Start the server in the background
        tokio::spawn(async move {
            if let Err(e) = server.await {
                println!("Server error: {}", e);
            }
        });

        // Wait for the callback result
        tokio::select! {
            result = self.wait_for_callback(oauth_state_clone) => {
                result
            }
            _ = tokio::time::sleep(timeout_duration) => {
                Err(anyhow!("Authentication timed out after 5 minutes"))
            }
        }
    }

    async fn wait_for_callback(&self, oauth_state: Arc<OAuthState>) -> Result<String> {
        loop {
            // Check if we received the auth result
            if let Some(result) = oauth_state.auth_result.read().await.as_ref() {
                return result
                    .as_ref()
                    .map(|s| s.clone())
                    .map_err(|e| anyhow!("{}", e));
            }

            // Sleep for a short time before checking again
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn handle_callback(
        Query(query): Query<AuthCallbackQuery>,
        oauth_state: Arc<OAuthState>,
    ) -> Html<&'static str> {
        println!("📨 Received authentication callback");

        let mut auth_result = oauth_state.auth_result.write().await;

        // Verify state parameter
        if let Some(state) = &query.state {
            if state != &oauth_state.state {
                *auth_result = Some(Err(anyhow!("Invalid state parameter").into()));
                return Html(SUCCESS_PAGE_ERROR);
            }
        } else {
            *auth_result = Some(Err(anyhow!("Missing state parameter").into()));
            return Html(SUCCESS_PAGE_ERROR);
        }

        // Check for errors
        if let Some(error) = &query.error {
            let error_desc = query
                .error_description
                .as_deref()
                .unwrap_or("Unknown error");
            *auth_result = Some(Err(
                anyhow!("OAuth error: {} - {}", error, error_desc).into()
            ));
            return Html(SUCCESS_PAGE_ERROR);
        }

        // Extract authorization code
        if let Some(code) = &query.code {
            *auth_result = Some(Ok(code.clone()));
            Html(SUCCESS_PAGE_SUCCESS)
        } else {
            *auth_result = Some(Err(anyhow!("Missing authorization code").into()));
            Html(SUCCESS_PAGE_ERROR)
        }
    }

    async fn exchange_code_for_tokens(
        &self,
        auth_code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<AuthTokens> {
        println!("🔄 Exchanging authorization code for tokens...");

        let client = reqwest::Client::new();
        let mut form_params = HashMap::new();
        form_params.insert("grant_type", "authorization_code");
        form_params.insert("client_id", CLIENT_ID);
        form_params.insert("code", auth_code);
        form_params.insert("redirect_uri", redirect_uri);
        form_params.insert("code_verifier", code_verifier);

        let response = client
            .post(OPENAI_TOKEN_URL)
            .form(&form_params)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Token exchange failed: {} - {}",
                status,
                error_text
            ));
        }

        let token_response: TokenResponse = response.json().await?;

        let expires_at = if let Some(expires_in) = token_response.expires_in {
            Utc::now() + Duration::seconds(expires_in as i64)
        } else {
            Utc::now() + Duration::hours(1) // Default 1 hour if not specified
        };

        Ok(AuthTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            expires_at,
            token_type: token_response.token_type,
        })
    }

    /// Start OAuth flow - returns the authorization URL for the user to visit
    pub async fn start_oauth_flow(&mut self) -> Result<String> {
        println!("🔑 Starting OAuth2 PKCE authentication flow...");

        // Generate PKCE parameters
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier)?;
        let state = Self::generate_state();

        // Store PKCE parameters for later use
        // In a real implementation, you'd store these securely
        // For now, we'll use the existing login method as a reference
        
        // Build authorization URL
        let auth_url = format!(
            "https://auth.openai.com/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            "your-client-id", // This should be configurable
            "http://127.0.0.1:8080/callback", // This should be configurable  
            "openid email profile",
            state,
            code_challenge
        );

        println!("OAuth authorization URL generated");
        Ok(auth_url)
    }

    /// Get current authentication status and user info
    pub async fn get_auth_status(&self) -> (bool, Option<String>) {
        let is_authenticated = self.is_authenticated().await;
        let user_email = if is_authenticated {
            // In a real implementation, you'd decode the JWT token to get user info
            Some("user@example.com".to_string()) // Mock email for now
        } else {
            None
        };
        (is_authenticated, user_email)
    }
}

// HTML pages for the callback server
const SUCCESS_PAGE_SUCCESS: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>MindLink - Authentication Successful</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; text-align: center; padding: 50px; background: #f0f2f5; }
        .container { max-width: 400px; margin: 0 auto; background: white; padding: 40px; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }
        .success { color: #059669; font-size: 24px; margin-bottom: 16px; }
        .message { color: #374151; margin-bottom: 24px; }
        .close-button { background: #10b981; color: white; border: none; padding: 12px 24px; border-radius: 8px; font-size: 16px; cursor: pointer; }
        .close-button:hover { background: #059669; }
    </style>
</head>
<body>
    <div class="container">
        <div class="success">✅ Authentication Successful</div>
        <div class="message">You have been successfully authenticated with OpenAI. You can now close this tab and return to MindLink.</div>
        <button class="close-button" onclick="window.close()">Close This Tab</button>
    </div>
    <script>setTimeout(() => window.close(), 3000);</script>
</body>
</html>
"#;

const SUCCESS_PAGE_ERROR: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>MindLink - Authentication Error</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; text-align: center; padding: 50px; background: #f0f2f5; }
        .container { max-width: 400px; margin: 0 auto; background: white; padding: 40px; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }
        .error { color: #dc2626; font-size: 24px; margin-bottom: 16px; }
        .message { color: #374151; margin-bottom: 24px; }
        .retry-button { background: #dc2626; color: white; border: none; padding: 12px 24px; border-radius: 8px; font-size: 16px; cursor: pointer; }
        .retry-button:hover { background: #b91c1c; }
    </style>
</head>
<body>
    <div class="container">
        <div class="error">❌ Authentication Error</div>
        <div class="message">There was an error during authentication. Please return to MindLink and try again.</div>
        <button class="retry-button" onclick="window.close()">Close This Tab</button>
    </div>
</body>
</html>
"#;
