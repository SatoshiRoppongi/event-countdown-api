use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterUserInfo {
    pub id: String,
    pub username: String,
    pub name: String,
    pub profile_image_url: Option<String>,
}

pub struct OAuthService;

impl OAuthService {
    pub fn get_google_config() -> Result<OAuthConfig, String> {
        Ok(OAuthConfig {
            client_id: env::var("GOOGLE_CLIENT_ID").map_err(|_| "GOOGLE_CLIENT_ID not set")?,
            client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .map_err(|_| "GOOGLE_CLIENT_SECRET not set")?,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://www.googleapis.com/oauth2/v4/token".to_string(),
            redirect_url: env::var("GOOGLE_REDIRECT_URL")
                .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/google/callback".to_string()),
        })
    }

    pub fn get_twitter_config() -> Result<OAuthConfig, String> {
        Ok(OAuthConfig {
            client_id: env::var("TWITTER_CLIENT_ID").map_err(|_| "TWITTER_CLIENT_ID not set")?,
            client_secret: env::var("TWITTER_CLIENT_SECRET")
                .map_err(|_| "TWITTER_CLIENT_SECRET not set")?,
            auth_url: "https://twitter.com/i/oauth2/authorize".to_string(),
            token_url: "https://api.twitter.com/2/oauth2/token".to_string(),
            redirect_url: env::var("TWITTER_REDIRECT_URL")
                .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/twitter/callback".to_string()),
        })
    }

    pub fn create_google_client(config: &OAuthConfig) -> Result<BasicClient, String> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone()).map_err(|e| format!("Invalid auth URL: {}", e))?,
            Some(
                TokenUrl::new(config.token_url.clone())
                    .map_err(|e| format!("Invalid token URL: {}", e))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_url.clone())
                .map_err(|e| format!("Invalid redirect URL: {}", e))?,
        );

        Ok(client)
    }

    pub fn get_google_auth_url(client: &BasicClient) -> (Url, CsrfToken) {
        client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url()
    }

    pub async fn get_google_user_info(access_token: &str) -> Result<GoogleUserInfo, String> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch user info: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Google API error: {}", response.status()));
        }

        let user_info: GoogleUserInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))?;

        Ok(user_info)
    }

    pub async fn get_twitter_user_info(access_token: &str) -> Result<TwitterUserInfo, String> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.twitter.com/2/users/me")
            .bearer_auth(access_token)
            .query(&[("user.fields", "profile_image_url")])
            .send()
            .await
            .map_err(|e| format!("Failed to fetch user info: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Twitter API error: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct TwitterResponse {
            data: TwitterUserInfo,
        }

        let twitter_response: TwitterResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))?;

        Ok(twitter_response.data)
    }
}