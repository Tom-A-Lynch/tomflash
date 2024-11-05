use tracing::{debug, info};
use serde::{Serialize, Deserialize};

use crate::{
    config::Config,
    utils::{Result, UtilError},
};

use super::ai::Client as AIClient;

pub struct SignificanceScorer {
    ai_client: AIClient,
    config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoringMetrics {
    novelty: f32,
    emotional_impact: f32,
    relevance: f32,
    persistence: f32,
}

impl SignificanceScorer {
    pub fn new(config: &Config) -> Result<Self> {
        let ai_client = AIClient::new(config)?;
        
        Ok(Self {
            ai_client,
            config: config.clone(),
        })
    }

    pub async fn score_memory(&self, content: &str) -> Result<f32> {
        let base_score = self.ai_client.calculate_significance(content).await?;
        
        // Apply additional heuristics
        let metrics = self.calculate_metrics(content);
        let final_score = self.combine_scores(base_score, &metrics);

        debug!(
            "Memory significance: {:.2} (base: {:.2}, novelty: {:.2}, impact: {:.2})",
            final_score, base_score, metrics.novelty, metrics.emotional_impact
        );

        Ok(final_score)
    }

    fn calculate_metrics(&self, content: &str) -> ScoringMetrics {
        // Heuristic scoring based on content analysis
        let word_count = content.split_whitespace().count();
        let unique_words: std::collections::HashSet<_> = content.split_whitespace().collect();
        
        let novelty = (unique_words.len() as f32 / word_count as f32).min(1.0);
        let emotional_words = count_emotional_words(content);
        let emotional_impact = (emotional_words as f32 / word_count as f32).min(1.0);
        
        ScoringMetrics {
            novelty,
            emotional_impact,
            relevance: 0.5, // Default mid-range for now
            persistence: 0.5,
        }
    }

    fn combine_scores(&self, base_score: f32, metrics: &ScoringMetrics) -> f32 {
        let weighted_score = 
            base_score * 0.4 +
            metrics.novelty * 0.2 +
            metrics.emotional_impact * 0.2 +
            metrics.relevance * 0.1 +
            metrics.persistence * 0.1;

        weighted_score.min(1.0).max(0.0)
    }
}

fn count_emotional_words(content: &str) -> usize {
    // Simple emotional word detection
    // In production, this would use a proper sentiment lexicon
    let emotional_indicators = [
        "love", "hate", "amazing", "terrible", "excited",
        "angry", "sad", "happy", "worried", "confident",
        "afraid", "proud", "disgusted", "surprised", "peaceful",
    ];

    content
        .to_lowercase()
        .split_whitespace()
        .filter(|word| emotional_indicators.contains(word))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_significance_scoring() {
        let config = Config::from_env().unwrap();
        let scorer = SignificanceScorer::new(&config).unwrap();

        let test_cases = [
            ("Just another normal day", 0.3),
            ("HOLY SHIT I JUST HAD THE MOST AMAZING REVELATION!", 0.8),
            ("I feel deeply connected to this profound insight", 0.6),
        ];

        for (content, expected_min) in test_cases {
            let score = scorer.score_memory(content).await.unwrap();
            assert!(
                score >= expected_min,
                "Score {} for '{}' below expected minimum {}",
                score, content, expected_min
            );
        }
    }
}