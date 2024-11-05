use reqwest::{Client as ReqwestClient, header};
use serde_json::json;
use tracing::{debug, error, info};
use crate::config::{Config, TwitterConfig}; // Add full Config


use super::{Result, XError, types::*};
use crate::config::TwitterConfig;

const API_BASE: &str = "https://api.twitter.com/2";

pub struct Client {
    http: ReqwestClient,
    config: TwitterConfig,
    auth_tokens: AuthTokens,
}

impl Client {
    pub fn new(config: &Config) -> Result<Self> {
        let twitter_config = &config.twitter_config;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("Bearer {}", &config.bearer_token))
                .map_err(|e| XError::AuthError(e.to_string()))?
        );

        let http = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| XError::NetworkError(e))?;

        Ok(Self {
            http,
            config: config.clone(),
            auth_tokens: AuthTokens::default(),
        })
    }

    pub async fn post_tweet(&self, content: &str) -> Result<Tweet> {
        let url = format!("{}/tweets", API_BASE);
        
        let response = self.http
            .post(&url)
            .json(&json!({
                "text": content
            }))
            .send()
            .await
            .map_err(XError::NetworkError)?;

        if !response.status().is_success() {
            let error = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(XError::ApiError(error));
        }

        let tweet: TweetResponse = response.json().await
            .map_err(|e| XError::ParseError(e.to_string()))?;

        Ok(tweet.data)
    }

    pub async fn reply_to_tweet(&self, content: &str, reply_to_id: &str) -> Result<Tweet> {
        let url = format!("{}/tweets", API_BASE);
        
        let response = self.http
            .post(&url)
            .json(&json!({
                "text": content,
                "reply": {
                    "in_reply_to_tweet_id": reply_to_id
                }
            }))
            .send()
            .await
            .map_err(XError::NetworkError)?;

        if !response.status().is_success() {
            let error = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(XError::ApiError(error));
        }

        let tweet: TweetResponse = response.json().await
            .map_err(|e| XError::ParseError(e.to_string()))?;

        Ok(tweet.data)
    }

    pub async fn get_home_timeline(&self, limit: usize) -> Result<Vec<Tweet>> {
        let url = format!("{}/tweets/search/recent", API_BASE);
        
        let response = self.http
            .get(&url)
            .query(&[
                ("max_results", limit.to_string()),
                ("tweet.fields", "created_at,author_id,conversation_id,in_reply_to_user_id".to_string()),
                ("expansions", "author_id,referenced_tweets.id".to_string()),
                ("user.fields", "username,name,description".to_string()),
            ])
            .send()
            .await
            .map_err(XError::NetworkError)?;

        if !response.status().is_success() {
            let error = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(XError::ApiError(error));
        }

        let timeline: TimelineResponse = response.json().await
            .map_err(|e| XError::ParseError(e.to_string()))?;

        Ok(timeline.data)
    }

    pub async fn follow_user(&self, username: &str) -> Result<bool> {
        let user = self.get_user_by_username(username).await?;
        let url = format!("{}/users/{}/following", API_BASE, self.auth_tokens.user_id);

        let response = self.http
            .post(&url)
            .json(&json!({
                "target_user_id": user.id
            }))
            .send()
            .await
            .map_err(XError::NetworkError)?;

        Ok(response.status().is_success())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let url = format!("{}/users/by/username/{}", API_BASE, username);
        
        let response = self.http
            .get(&url)
            .send()
            .await
            .map_err(XError::NetworkError)?;

        if !response.status().is_success() {
            let error = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(XError::ApiError(error));
        }

        let user_response: UserResponse = response.json().await
            .map_err(|e| XError::ParseError(e.to_string()))?;

        Ok(user_response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        HttpClient {}
        async trait HttpClient {
            async fn post(&self, url: &str, json: serde_json::Value) -> Result<reqwest::Response>;
            async fn get(&self, url: &str) -> Result<reqwest::Response>;
        }
    }

    #[tokio::test]
    async fn test_post_tweet() {
        let config = TwitterConfig {
            api_key: "test".to_string(),
            api_secret: "test".to_string(),
            access_token: "test".to_string(),
            access_secret: "test".to_string(),
            bearer_token: "test".to_string(),
        };

        let mut mock_http = MockHttpClient::new();
        mock_http
            .expect_post()
            .with(eq("/tweets"), eq(json!({"text": "test tweet"})))
            .returning(|_, _| {
                Ok(reqwest::Response::new(200, json!({
                    "data": {
                        "id": "123",
                        "text": "test tweet"
                    }
                })))
            });

        let client = Client::new(&config).unwrap();
        let result = client.post_tweet("test tweet").await;
        assert!(result.is_ok());
    }
}