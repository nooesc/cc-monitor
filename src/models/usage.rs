use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub cwd: Option<String>,
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub model: String,
    pub usage: TokenUsage,
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    #[serde(rename = "costUSD")]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

impl TokenUsage {
    pub fn total_input(&self) -> u64 {
        self.input_tokens + self.cache_creation_input_tokens + self.cache_read_input_tokens
    }

    pub fn total(&self) -> u64 {
        self.total_input() + self.output_tokens
    }

    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub tokens: TokenUsage,
    pub total_cost: f64,
    pub models_used: HashSet<String>,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionUsage {
    pub session_id: String,
    pub project_path: String,
    pub tokens: TokenUsage,
    pub total_cost: f64,
    pub last_activity: DateTime<Utc>,
    pub models_used: HashSet<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyUsage {
    pub month: String, // YYYY-MM format
    pub tokens: TokenUsage,
    pub total_cost: f64,
    pub models_used: HashSet<String>,
    pub daily_breakdown: Vec<DailyUsage>,
}

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_tokens: TokenUsage,
    pub total_cost: f64,
    pub sessions: Vec<SessionUsage>,
    pub daily: Vec<DailyUsage>,
    pub monthly: Vec<MonthlyUsage>,
}