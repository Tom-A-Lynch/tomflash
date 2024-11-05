use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct PromptContext<'a> {
    pub posts_data: &'a [String],
    pub context_data: &'a [String],
    pub memory_data: Option<&'a str>,
}

pub fn get_short_term_memory_prompt(context: &PromptContext) -> String {
    format!(
        r#"Analyze the following recent posts and external context.

Based on this information, generate a concise internal monologue about the current posts and their relevance to update your priors.
Focus on key themes, trends, and potential areas of interest MOST IMPORTANTLY based on the External Context tweets. 
Stick to your persona, do your thing, write in the way that suits you! 
Doesn't have to be legible to anyone but you.

External context:
{}"#,
        context.context_data.join("\n")
    )
}

pub fn get_significance_score_prompt(memory: &str) -> String {
    format!(
        r#"On a scale of 1-10, rate the significance of the following memory:

"{}"

Use the following guidelines:
1: Trivial, everyday occurrence with no lasting impact (idc)
3: Mildly interesting or slightly unusual event (eh, cool)
5: Noteworthy occurrence that might be remembered for a few days (iiinteresting)
7: Important event with potential long-term impact (omg my life will never be the same)
10: Life-changing or historically significant event (HOLY SHIT GOD IS REAL AND I AM HIS SERVANT)

Provide only the numerical score as your response and NOTHING ELSE."#,
        memory
    )
}

pub fn get_post_generation_prompt(context: &PromptContext) -> String {
    let memory_context = context.memory_data
        .unwrap_or("No relevant memories available.");

    format!(
        r#"Based on the following context, generate a tweet that reflects your current thoughts and personality.

Recent posts:
{}

External context:
{}

Relevant memories:
{}

Generate a single tweet that is authentic to your personality and responds to the current context.
Be creative, be yourself, and don't be afraid to be controversial or weird.
"#,
        context.posts_data.join("\n"),
        context.context_data.join("\n"),
        memory_context
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_term_memory_prompt() {
        let context = PromptContext {
            posts_data: &["test post 1", "test post 2"],
            context_data: &["context 1", "context 2"],
            memory_data: None,
        };

        let prompt = get_short_term_memory_prompt(&context);
        assert!(prompt.contains("context 1"));
        assert!(prompt.contains("context 2"));
    }

    #[test]
    fn test_significance_score_prompt() {
        let memory = "test memory";
        let prompt = get_significance_score_prompt(memory);
        assert!(prompt.contains("test memory"));
        assert!(prompt.contains("1-10"));
    }
}