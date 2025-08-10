use anyhow::{Result, Context};
use chrono::{Datelike, DateTime, Utc};
use glob::glob;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, warn, info};

use crate::models::{UsageEntry, DailyUsage, SessionUsage, MonthlyUsage, TokenUsage, UsageStats, PricingData};

pub struct DataLoader {
    claude_paths: Vec<PathBuf>,
    pricing: PricingData,
}

impl DataLoader {
    pub fn new() -> Result<Self> {
        let claude_paths = Self::find_claude_paths()?;
        if claude_paths.is_empty() {
            anyhow::bail!("No Claude data directories found");
        }
        
        Ok(Self {
            claude_paths,
            pricing: PricingData::new(),
        })
    }
    
    fn find_claude_paths() -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        
        // Check environment variable first
        if let Ok(env_paths) = std::env::var("CLAUDE_CONFIG_DIR") {
            for path_str in env_paths.split(',') {
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    let projects_path = path.join("projects");
                    if projects_path.exists() {
                        paths.push(path);
                    }
                }
            }
        }
        
        // If no env paths, check default locations
        if paths.is_empty() {
            if let Some(home) = directories::BaseDirs::new() {
                // New location: ~/.config/claude
                let config_path = home.config_dir().join("claude");
                if config_path.join("projects").exists() {
                    paths.push(config_path);
                }
                
                // Old location: ~/.claude
                let old_path = home.home_dir().join(".claude");
                if old_path.join("projects").exists() {
                    paths.push(old_path);
                }
            }
        }
        
        Ok(paths)
    }
    
    pub fn load_all_usage(&self) -> Result<UsageStats> {
        let mut all_entries = Vec::new();
        
        for claude_path in &self.claude_paths {
            let pattern = claude_path.join("projects").join("**/*.jsonl");
            let pattern_str = pattern.to_str()
                .context("Invalid path")?;
            
            for entry in glob(pattern_str)? {
                match entry {
                    Ok(path) => {
                        debug!("Loading file: {:?}", path);
                        let entries = self.load_jsonl_file(&path)?;
                        all_entries.extend(entries);
                    }
                    Err(e) => warn!("Error reading path: {}", e),
                }
            }
        }
        
        self.aggregate_usage(all_entries)
    }
    
    fn load_jsonl_file(&self, path: &Path) -> Result<Vec<UsageEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        
        // Extract session info from path: projects/{project}/{sessionId}.jsonl
        let session_id = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        
        for (line_num, line) in reader.lines().enumerate() {
            match line {
                Ok(json_str) => {
                    // Skip empty lines and incomplete JSON
                    if json_str.trim().is_empty() {
                        continue;
                    }
                    
                    // Validate that the line is complete JSON (starts with { and ends with })
                    let trimmed = json_str.trim();
                    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
                        debug!("Skipping incomplete JSON line {} in {:?}", line_num + 1, path);
                        continue;
                    }
                    
                    match serde_json::from_str::<UsageEntry>(&json_str) {
                        Ok(mut entry) => {
                            // Fill in session_id if missing
                            if entry.session_id.is_none() {
                                entry.session_id = session_id.clone();
                            }
                            entries.push(entry);
                        }
                        Err(e) => {
                            debug!("Failed to parse line {} in {:?}: {}", line_num + 1, path, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading line {} in {:?}: {}", line_num + 1, path, e);
                }
            }
        }
        
        Ok(entries)
    }
    
    fn aggregate_usage(&self, mut entries: Vec<UsageEntry>) -> Result<UsageStats> {
        // Sort entries by timestamp to ensure consistent processing order
        entries.sort_by_key(|e| e.timestamp);
        
        let mut daily_map: BTreeMap<chrono::NaiveDate, DailyUsage> = BTreeMap::new();
        let mut session_map: BTreeMap<String, SessionUsage> = BTreeMap::new();
        let mut monthly_map: BTreeMap<String, MonthlyUsage> = BTreeMap::new();
        let mut total_tokens = TokenUsage::default();
        let mut total_cost = 0.0;
        
        // Detect resumed sessions to avoid double-counting cache tokens
        let session_chains = self.detect_resumed_sessions(&entries);
        
        // Track maximum cache seen per session chain
        let mut chain_cache_max: BTreeMap<usize, (u64, u64)> = BTreeMap::new();
        
        for entry in entries {
            let date = entry.timestamp.date_naive();
            let month = format!("{:04}-{:02}", date.year(), date.month());
            let session_id = entry.session_id.clone().unwrap_or_else(|| "unknown".to_string());
            
            // Find which session chain this belongs to
            let chain_idx = session_chains.iter()
                .position(|chain| chain.contains(&session_id));
            
            // Adjust usage for resumed sessions to avoid double-counting cache
            let mut adjusted_usage = entry.message.usage.clone();
            
            if let Some(idx) = chain_idx {
                let (max_cache_read, max_cache_creation) = chain_cache_max.entry(idx)
                    .or_insert((0, 0));
                
                // Only count incremental cache, not the full amount
                let incremental_cache_read = adjusted_usage.cache_read_input_tokens
                    .saturating_sub(*max_cache_read);
                let incremental_cache_creation = adjusted_usage.cache_creation_input_tokens
                    .saturating_sub(*max_cache_creation);
                
                adjusted_usage.cache_read_input_tokens = incremental_cache_read;
                adjusted_usage.cache_creation_input_tokens = incremental_cache_creation;
                
                // Update max cache seen
                *max_cache_read = (*max_cache_read).max(entry.message.usage.cache_read_input_tokens);
                *max_cache_creation = (*max_cache_creation).max(entry.message.usage.cache_creation_input_tokens);
            }
            
            // Calculate cost with adjusted usage
            let cost = if let Some(cost) = entry.message.cost_usd {
                cost
            } else {
                self.pricing.calculate_cost(&entry.message.model, &adjusted_usage)
            };
            
            // Update totals with adjusted usage
            total_tokens.add(&adjusted_usage);
            total_cost += cost;
            
            // Update daily stats
            let daily = daily_map.entry(date).or_insert_with(|| DailyUsage {
                date,
                tokens: TokenUsage::default(),
                total_cost: 0.0,
                models_used: HashSet::new(),
                session_count: 0,
            });
            daily.tokens.add(&adjusted_usage);
            daily.total_cost += cost;
            daily.models_used.insert(entry.message.model.clone());
            
            // Update session stats
            let session = session_map.entry(session_id.clone()).or_insert_with(|| SessionUsage {
                session_id: session_id.clone(),
                project_path: entry.cwd.clone().unwrap_or_else(|| "unknown".to_string()),
                tokens: TokenUsage::default(),
                total_cost: 0.0,
                last_activity: entry.timestamp,
                models_used: HashSet::new(),
            });
            session.tokens.add(&adjusted_usage);
            session.total_cost += cost;
            session.last_activity = session.last_activity.max(entry.timestamp);
            session.models_used.insert(entry.message.model.clone());
            
            // Update monthly stats
            let monthly = monthly_map.entry(month.clone()).or_insert_with(|| MonthlyUsage {
                month: month.clone(),
                tokens: TokenUsage::default(),
                total_cost: 0.0,
                models_used: HashSet::new(),
                daily_breakdown: Vec::new(),
            });
            monthly.tokens.add(&adjusted_usage);
            monthly.total_cost += cost;
            monthly.models_used.insert(entry.message.model);
        }
        
        // Convert maps to sorted vectors
        let mut daily: Vec<_> = daily_map.into_iter().map(|(_, v)| v).collect();
        daily.sort_by_key(|d| d.date);
        
        let mut sessions: Vec<_> = session_map.into_iter().map(|(_, v)| v).collect();
        sessions.sort_by_key(|s| std::cmp::Reverse(s.last_activity));
        
        let mut monthly: Vec<_> = monthly_map.into_iter().map(|(_, v)| v).collect();
        monthly.sort_by_key(|m| m.month.clone());
        
        // Add daily breakdown to monthly stats
        for month_usage in &mut monthly {
            let month_year: Vec<_> = month_usage.month.split('-').collect();
            let year: i32 = month_year[0].parse()?;
            let month: u32 = month_year[1].parse()?;
            
            month_usage.daily_breakdown = daily.iter()
                .filter(|d| d.date.year() == year && d.date.month() == month)
                .cloned()
                .collect();
        }
        
        Ok(UsageStats {
            total_tokens,
            total_cost,
            sessions,
            daily,
            monthly,
        })
    }
    
    /// Detect resumed sessions based on timing and project
    /// Sessions that start within 10 minutes of each other in the same project
    /// are likely resumed sessions sharing the same cache
    fn detect_resumed_sessions(&self, entries: &[UsageEntry]) -> Vec<Vec<String>> {
        // Build session info
        let mut session_times: BTreeMap<String, (DateTime<Utc>, DateTime<Utc>, String)> = BTreeMap::new();
        
        for entry in entries {
            if let Some(session_id) = &entry.session_id {
                let project = entry.cwd.clone().unwrap_or_else(|| "unknown".to_string());
                let times = session_times.entry(session_id.clone())
                    .or_insert((entry.timestamp, entry.timestamp, project.clone()));
                times.0 = times.0.min(entry.timestamp);
                times.1 = times.1.max(entry.timestamp);
            }
        }
        
        // Group sessions into chains
        let mut chains: Vec<Vec<String>> = Vec::new();
        let mut processed = HashSet::new();
        
        for (session_id, (start, end, project)) in &session_times {
            if processed.contains(session_id) {
                continue;
            }
            
            let mut chain = vec![session_id.clone()];
            processed.insert(session_id.clone());
            
            // Find sessions that might be resumptions
            for (other_id, (other_start, _, other_project)) in &session_times {
                if processed.contains(other_id) || other_project != project {
                    continue;
                }
                
                // Check if this session starts shortly after the current chain ends
                let gap_minutes = other_start.signed_duration_since(*end).num_minutes();
                if gap_minutes >= 0 && gap_minutes <= 10 {
                    chain.push(other_id.clone());
                    processed.insert(other_id.clone());
                }
            }
            
            if chain.len() > 1 {
                info!("Detected resumed session chain with {} sessions", chain.len());
            }
            
            chains.push(chain);
        }
        
        chains
    }
}