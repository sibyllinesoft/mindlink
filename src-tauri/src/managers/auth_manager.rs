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
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde_json::Value;

use crate::error::{MindLinkError, MindLinkResult};
use crate::{auth_error, log_error, log_info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub api_key: String,
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

// ChatGPT OAuth configuration using Codex CLI client ID
const CHATGPT_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const CHATGPT_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann"; // Codex CLI's client ID for ChatGPT access
const SCOPE: &str = "openid profile email offline_access";
const REDIRECT_PORT: u16 = 1455; // Required port for Codex CLI flow
const CHATGPT_API_URL: &str = "https://chatgpt.com/backend-api/codex/responses";

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
            .post(CHATGPT_TOKEN_URL)
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
            id_token: current_tokens.id_token.clone(),
            expires_at: Utc::now()
                + Duration::seconds(refresh_response.expires_in.unwrap_or(3600) as i64),
            token_type: refresh_response.token_type,
            account_id: current_tokens.account_id.clone(),
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
        println!("🔐 Starting ChatGPT OAuth2 PKCE authentication flow...");

        // Generate PKCE parameters
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier)?;
        let state = Self::generate_state();

        // Use fixed port for Codex CLI compatibility
        let redirect_uri = format!("http://localhost:{}/auth/callback", REDIRECT_PORT);
        let listener = TcpListener::bind(format!("127.0.0.1:{}", REDIRECT_PORT)).await?;

        println!(
            "📡 Starting local callback server on port {}",
            REDIRECT_PORT
        );

        // Prepare OAuth state
        let oauth_state = Arc::new(OAuthState {
            code_verifier: code_verifier.clone(),
            state: state.clone(),
            auth_result: Arc::new(RwLock::new(None)),
        });

        // Build authorization URL for ChatGPT
        let auth_url = Self::build_chatgpt_auth_url(&redirect_uri, &code_challenge, &state)?;
        println!("🌐 Opening browser for ChatGPT authentication...");

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
            .exchange_code_for_chatgpt_tokens(&auth_code, &code_verifier, &redirect_uri)
            .await?;

        // Store tokens
        self.tokens = Some(tokens);
        self.save_tokens().await?;

        println!("✅ ChatGPT authentication successful!");
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
            .post(CHATGPT_TOKEN_URL)
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
            id_token: tokens.id_token.clone(),
            expires_at,
            token_type: refresh_response.token_type,
            account_id: tokens.account_id.clone(),
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

    /// Get the current authentication tokens
    pub fn get_tokens(&self) -> Option<&AuthTokens> {
        self.tokens.as_ref()
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
                            id_token: "".to_string(), // Empty for migrated tokens
                            expires_at: old_tokens.expires_at,
                            token_type: "Bearer".to_string(), // Default for OpenAI
                            account_id: "".to_string(), // Empty for migrated tokens
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

    fn build_chatgpt_auth_url(redirect_uri: &str, code_challenge: &str, state: &str) -> Result<String> {
        let mut url = Url::parse(CHATGPT_AUTH_URL)?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", CLIENT_ID)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", SCOPE)
            .append_pair("state", state)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("id_token_add_organizations", "true")
            .append_pair("codex_cli_simplified_flow", "true"); // Critical for Codex CLI access
        Ok(url.to_string())
    }

    async fn open_browser(url: &str) -> Result<()> {
        // Use Tauri's opener plugin for better compatibility
        println!("🌐 Opening OAuth URL in default browser: {}", url);
        
        // Use tauri_plugin_opener for cross-platform URL opening
        tauri_plugin_opener::open_url(url, None::<&str>)
            .map_err(|e| anyhow!("Failed to open browser: {}", e))?;
        
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
            "/auth/callback",
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
        println!("📨 Received ChatGPT authentication callback");

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
                anyhow!("ChatGPT OAuth error: {} - {}", error, error_desc).into()
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

    async fn exchange_code_for_chatgpt_tokens(
        &self,
        auth_code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<AuthTokens> {
        println!("🔄 Exchanging authorization code for ChatGPT tokens...");

        let client = reqwest::Client::new();
        let mut form_params = HashMap::new();
        form_params.insert("grant_type", "authorization_code");
        form_params.insert("client_id", CLIENT_ID);
        form_params.insert("code", auth_code);
        form_params.insert("redirect_uri", redirect_uri);
        form_params.insert("code_verifier", code_verifier);

        let response = client
            .post(CHATGPT_TOKEN_URL)
            .form(&form_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "ChatGPT token exchange failed: {} - {}",
                status,
                error_text
            ));
        }

        let token_response: TokenResponse = response.json().await?;

        // Extract account ID from ID token
        let account_id = Self::extract_account_id_from_id_token(&token_response.id_token)?;
        
        let expires_at = if let Some(expires_in) = token_response.expires_in {
            Utc::now() + Duration::seconds(expires_in as i64)
        } else {
            Utc::now() + Duration::hours(1) // Default 1 hour if not specified
        };

        Ok(AuthTokens {
            access_token: token_response.access_token,
            id_token: token_response.id_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            expires_at,
            token_type: token_response.token_type,
            account_id,
        })
    }

    /// Extract chatgpt_account_id from JWT ID token
    fn extract_account_id_from_id_token(id_token: &str) -> Result<String> {
        // Decode JWT without verification (we trust the source since it came from OAuth)
        let header = decode_header(id_token)
            .map_err(|e| anyhow!("Failed to decode JWT header: {}", e))?;

        // Use unsafe decode since we're just extracting claims
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.validate_aud = false; // Disable audience validation
        validation.validate_exp = false; // Disable expiration validation
        validation.validate_nbf = false; // Disable not-before validation
        
        let token_data = decode::<Value>(
            id_token,
            &DecodingKey::from_secret(&[]), // Empty key since verification is disabled
            &validation,
        )
        .map_err(|e| anyhow!("Failed to decode JWT: {}", e))?;

        // Extract chatgpt_account_id from auth claims
        let auth_claims = token_data.claims
            .get("https://api.openai.com/auth")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("Missing auth claims in ID token"))?;

        let account_id = auth_claims
            .get("chatgpt_account_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing chatgpt_account_id in auth claims"))?;

        println!("✅ Extracted ChatGPT account ID: {}", account_id);
        Ok(account_id.to_string())
    }

    /// Make authenticated ChatGPT API request
    pub async fn make_chatgpt_request(&self, messages: &[serde_json::Value]) -> Result<String> {
        let tokens = self.tokens.as_ref()
            .ok_or_else(|| anyhow!("No authentication tokens available"))?;

        let client = reqwest::Client::new();
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let request_body = serde_json::json!({
            "messages": messages,
            "model": "gpt-4", 
            "stream": false
        });

        let response = client
            .post(CHATGPT_API_URL)
            .header("Authorization", format!("Bearer {}", tokens.access_token))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("chatgpt-account-id", &tokens.account_id)
            .header("OpenAI-Beta", "responses=experimental") // Critical header
            .header("session_id", session_id)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("ChatGPT API request failed: {} - {}", status, error_text));
        }

        let response_text = response.text().await?;
        Ok(response_text)
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
    <title>MindLink - ChatGPT Authentication Successful</title>
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
        <div class="success">✅ ChatGPT Authentication Successful</div>
        <div class="message">You have been successfully authenticated with ChatGPT. You can now close this tab and return to MindLink.</div>
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
    <title>MindLink - ChatGPT Authentication Error</title>
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
        <div class="error">❌ ChatGPT Authentication Error</div>
        <div class="message">There was an error during ChatGPT authentication. Please return to MindLink and try again.</div>
        <button class="retry-button" onclick="window.close()">Close This Tab</button>
    </div>
</body>
</html>
"#;
