use chrono::{Local, Duration, Datelike};
use crate::models::{UsageStats, DailyUsage, TokenUsage};

pub struct App {
    pub stats: UsageStats,
    pub selected_tab: Tab,
    pub selected_index: usize,
    pub should_quit: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Overview,
    Daily,
    Sessions,
    Monthly,
}

impl App {
    pub fn new(stats: UsageStats) -> Self {
        Self {
            stats,
            selected_tab: Tab::Overview,
            selected_index: 0,
            should_quit: false,
        }
    }
    
    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Overview => Tab::Daily,
            Tab::Daily => Tab::Sessions,
            Tab::Sessions => Tab::Monthly,
            Tab::Monthly => Tab::Overview,
        };
        self.selected_index = 0;
    }
    
    pub fn previous_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Overview => Tab::Monthly,
            Tab::Daily => Tab::Overview,
            Tab::Sessions => Tab::Daily,
            Tab::Monthly => Tab::Sessions,
        };
        self.selected_index = 0;
    }
    
    pub fn next_item(&mut self) {
        let max_index = match self.selected_tab {
            Tab::Overview => 0,
            Tab::Daily => self.stats.daily.len().saturating_sub(1),
            Tab::Sessions => self.stats.sessions.len().saturating_sub(1),
            Tab::Monthly => self.stats.monthly.len().saturating_sub(1),
        };
        
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }
    
    pub fn previous_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    pub fn get_today_stats(&self) -> Option<&DailyUsage> {
        let today = Local::now().date_naive();
        self.stats.daily.iter().find(|d| d.date == today)
    }
    
    pub fn get_week_stats(&self) -> (TokenUsage, f64) {
        let today = Local::now().date_naive();
        let week_ago = today - Duration::days(7);
        
        self.stats.daily.iter()
            .filter(|d| d.date > week_ago)
            .fold((TokenUsage::default(), 0.0), |(mut tokens, cost), d| {
                tokens.add(&d.tokens);
                (tokens, cost + d.total_cost)
            })
    }
    
    pub fn get_month_stats(&self) -> Option<&crate::models::MonthlyUsage> {
        let today = Local::now().date_naive();
        let current_month = format!("{:04}-{:02}", today.year(), today.month());
        self.stats.monthly.iter().find(|m| m.month == current_month)
    }
}