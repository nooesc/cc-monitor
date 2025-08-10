use anyhow::{Result, Context};
use chrono::Datelike;
use glob::glob;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

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
        
        for entry in entries {
            let date = entry.timestamp.date_naive();
            let month = format!("{:04}-{:02}", date.year(), date.month());
            let session_id = entry.session_id.clone().unwrap_or_else(|| "unknown".to_string());
            
            // Calculate cost
            // IMPORTANT: Only calculate cost if it's not already in the JSONL
            // If cost_usd is present, use it. Otherwise calculate it ONCE.
            let cost = if let Some(cost) = entry.message.cost_usd {
                cost
            } else {
                // Only calculate for models we have pricing for
                // This prevents double-counting when re-reading files
                self.pricing.calculate_cost(&entry.message.model, &entry.message.usage)
            };
            
            // Update totals
            total_tokens.add(&entry.message.usage);
            total_cost += cost;
            
            // Update daily stats
            let daily = daily_map.entry(date).or_insert_with(|| DailyUsage {
                date,
                tokens: TokenUsage::default(),
                total_cost: 0.0,
                models_used: HashSet::new(),
                session_count: 0,
            });
            daily.tokens.add(&entry.message.usage);
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
            session.tokens.add(&entry.message.usage);
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
            monthly.tokens.add(&entry.message.usage);
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
}