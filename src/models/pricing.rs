use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
    #[serde(default)]
    pub cache_creation_input_token_cost: f64,
    #[serde(default)]
    pub cache_read_input_token_cost: f64,
}

impl ModelPricing {
    pub fn calculate_cost(&self, tokens: &crate::models::TokenUsage) -> f64 {
        let input_cost = tokens.input_tokens as f64 * self.input_cost_per_token;
        let output_cost = tokens.output_tokens as f64 * self.output_cost_per_token;
        let cache_creation_cost = tokens.cache_creation_input_tokens as f64 * self.cache_creation_input_token_cost;
        let cache_read_cost = tokens.cache_read_input_tokens as f64 * self.cache_read_input_token_cost;
        
        input_cost + output_cost + cache_creation_cost + cache_read_cost
    }
}

pub struct PricingData {
    models: HashMap<String, ModelPricing>,
}

impl PricingData {
    pub fn new() -> Self {
        let mut models = HashMap::new();
        
        // Claude 3.5 Sonnet pricing (per million tokens)
        models.insert("claude-3-5-sonnet-20241022".to_string(), ModelPricing {
            input_cost_per_token: 3.0 / 1_000_000.0,
            output_cost_per_token: 15.0 / 1_000_000.0,
            cache_creation_input_token_cost: 3.75 / 1_000_000.0,
            cache_read_input_token_cost: 0.30 / 1_000_000.0,
        });
        
        // Claude 3.5 Haiku pricing
        models.insert("claude-3-5-haiku-20241022".to_string(), ModelPricing {
            input_cost_per_token: 1.0 / 1_000_000.0,
            output_cost_per_token: 5.0 / 1_000_000.0,
            cache_creation_input_token_cost: 1.25 / 1_000_000.0,
            cache_read_input_token_cost: 0.10 / 1_000_000.0,
        });
        
        // Claude 3 Opus pricing
        models.insert("claude-3-opus-20240229".to_string(), ModelPricing {
            input_cost_per_token: 15.0 / 1_000_000.0,
            output_cost_per_token: 75.0 / 1_000_000.0,
            cache_creation_input_token_cost: 18.75 / 1_000_000.0,
            cache_read_input_token_cost: 1.50 / 1_000_000.0,
        });
        
        Self { models }
    }
    
    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model)
    }
    
    pub fn calculate_cost(&self, model: &str, tokens: &crate::models::TokenUsage) -> f64 {
        self.get_pricing(model)
            .map(|p| p.calculate_cost(tokens))
            .unwrap_or(0.0)
    }
}