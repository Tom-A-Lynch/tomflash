use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::error;

use super::{Result, UtilError};

/// Formats conversation data for LLM processing
pub fn format_conversation_for_llm(data: &Value, tweet_id: &str) -> Result<String> {
    let tweets = data.get("globalObjects")
        .and_then(|obj| obj.get("tweets"))
        .ok_or_else(|| UtilError::JsonError(serde_json::Error::custom("Missing tweets object")))?;

    let users = data.get("globalObjects")
        .and_then(|obj| obj.get("users"))
        .ok_or_else(|| UtilError::JsonError(serde_json::Error::custom("Missing users object")))?;

    let mut processed_ids = std::collections::HashSet::new();
    let conversation = get_conversation_chain(tweets, users, tweet_id, &mut processed_ids)?;

    format_conversation_output(&conversation)
}

#[derive(Debug, Serialize, Deserialize)]
struct ConversationItem {
    id: String,
    username: String,
    text: String,
    reply_to: Option<String>,
}

fn get_conversation_chain(
    tweets: &Value,
    users: &Value,
    current_id: &str,
    processed_ids: &mut std::collections::HashSet<String>,
) -> Result<Vec<ConversationItem>> {
    if processed_ids.contains(current_id) {
        return Ok(vec![]);
    }

    processed_ids.insert(current_id.to_string());

    let current_tweet = tweets.get(current_id)
        .ok_or_else(|| UtilError::JsonError(serde_json::Error::custom("Tweet not found")))?;

    let user_id = current_tweet["user_id"].as_str()
        .ok_or_else(|| UtilError::JsonError(serde_json::Error::custom("User ID not found")))?;

    let user = users.get(user_id)
        .ok_or_else(|| UtilError::JsonError(serde_json::Error::custom("User not found")))?;

    let username = user["screen_name"].as_str()
        .map(|name| format!("@{}", name))
        .unwrap_or_else(|| "Unknown User".to_string());

    let mut chain = vec![ConversationItem {
        id: current_id.to_string(),
        username,
        text: current_tweet["full_text"].as_str()
            .unwrap_or("").to_string(),
        reply_to: current_tweet["in_reply_to_status_id_str"]
            .as_str()
            .map(String::from),
    }];

    // Get replies
    for (id, tweet) in tweets.as_object().unwrap() {
        if let Some(reply_to) = tweet["in_reply_to_status_id_str"].as_str() {
            if reply_to == current_id {
                let mut replies = get_conversation_chain(tweets, users, id, processed_ids)?;
                chain.extend(replies);
            }
        }
    }

    Ok(chain)
}

fn format_conversation_output(conversation: &[ConversationItem]) -> Result<String> {
    let mut output = String::new();
    
    for item in conversation {
        output.push_str(&format!("{}: {}\n", item.username, item.text));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_formatting() {
        let sample_data = json!({
            "globalObjects": {
                "tweets": {
                    "1": {
                        "full_text": "Hello world",
                        "user_id": "123",
                        "in_reply_to_status_id_str": null
                    },
                    "2": {
                        "full_text": "Hello back",
                        "user_id": "456",
                        "in_reply_to_status_id_str": "1"
                    }
                },
                "users": {
                    "123": {
                        "screen_name": "user1"
                    },
                    "456": {
                        "screen_name": "user2"
                    }
                }
            }
        });

        let result = format_conversation_for_llm(&sample_data, "1");
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("@user1"));
        assert!(formatted.contains("@user2"));
    }
}