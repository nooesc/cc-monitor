use anyhow::Result;
use chrono::{Local, Duration, Datelike};
use crate::data_loader::DataLoader;
use crate::models::TokenUsage;

pub fn show_status(detailed: bool, json: bool) -> Result<()> {
    let loader = DataLoader::new()?;
    let stats = loader.load_all_usage()?;
    
    if json {
        print_json_status(&stats)?;
    } else {
        print_text_status(&stats, detailed)?;
    }
    
    Ok(())
}

fn print_text_status(stats: &crate::models::UsageStats, detailed: bool) -> Result<()> {
    println!("🤖 Claude Code Usage Status\n");
    println!("═══════════════════════════════════════════");
    
    // Today's usage
    let today = Local::now().date_naive();
    let today_usage = stats.daily.iter()
        .find(|d| d.date == today);
    
    if let Some(usage) = today_usage {
        println!("📅 Today ({}):", today.format("%Y-%m-%d"));
        println!("   Tokens: {} input / {} output", 
            format_number(usage.tokens.input_tokens),
            format_number(usage.tokens.output_tokens));
        if usage.tokens.cache_read_input_tokens > 0 || usage.tokens.cache_creation_input_tokens > 0 {
            println!("   Cache:  {} read / {} created",
                format_number(usage.tokens.cache_read_input_tokens),
                format_number(usage.tokens.cache_creation_input_tokens));
        }
        println!("   Cost:   ${:.2}", usage.total_cost);
        println!();
    }
    
    // Last 7 days
    let week_ago = today - Duration::days(7);
    let week_stats = stats.daily.iter()
        .filter(|d| d.date > week_ago)
        .fold((TokenUsage::default(), 0.0), |(mut tokens, cost), d| {
            tokens.add(&d.tokens);
            (tokens, cost + d.total_cost)
        });
    
    println!("📊 Last 7 Days:");
    println!("   Tokens: {} total", format_number(week_stats.0.total()));
    println!("   Cost:   ${:.2}", week_stats.1);
    println!();
    
    // Current month
    let current_month = format!("{:04}-{:02}", today.year(), today.month());
    let month_usage = stats.monthly.iter()
        .find(|m| m.month == current_month);
    
    if let Some(usage) = month_usage {
        println!("📈 This Month ({}):", current_month);
        println!("   Tokens: {} total", format_number(usage.tokens.total()));
        println!("   Cost:   ${:.2}", usage.total_cost);
        let models: Vec<String> = usage.models_used.iter().cloned().collect();
        println!("   Models: {}", models.join(", "));
        println!();
    }
    
    // Total stats
    println!("💰 All Time:");
    println!("   Tokens: {} total", format_number(stats.total_tokens.total()));
    println!("   Cost:   ${:.2}", stats.total_cost);
    println!("   Sessions: {}", stats.sessions.len());
    
    if detailed {
        println!("\n📝 Recent Sessions:");
        for session in stats.sessions.iter().take(5) {
            println!("   {} - {} tokens, ${:.2}",
                session.last_activity.format("%Y-%m-%d %H:%M"),
                format_number(session.tokens.total()),
                session.total_cost);
        }
    }
    
    println!("═══════════════════════════════════════════");
    
    Ok(())
}

fn print_json_status(stats: &crate::models::UsageStats) -> Result<()> {
    let today = Local::now().date_naive();
    let week_ago = today - Duration::days(7);
    let current_month = format!("{:04}-{:02}", today.year(), today.month());
    
    let today_usage = stats.daily.iter()
        .find(|d| d.date == today);
    
    let week_stats = stats.daily.iter()
        .filter(|d| d.date > week_ago)
        .fold((TokenUsage::default(), 0.0), |(mut tokens, cost), d| {
            tokens.add(&d.tokens);
            (tokens, cost + d.total_cost)
        });
    
    let month_usage = stats.monthly.iter()
        .find(|m| m.month == current_month);
    
    let output = serde_json::json!({
        "today": today_usage.map(|u| {
            serde_json::json!({
                "date": u.date.to_string(),
                "tokens": {
                    "input": u.tokens.input_tokens,
                    "output": u.tokens.output_tokens,
                    "cache_read": u.tokens.cache_read_input_tokens,
                    "cache_creation": u.tokens.cache_creation_input_tokens,
                    "total": u.tokens.total()
                },
                "cost": u.total_cost,
                "models": u.models_used
            })
        }),
        "last_7_days": {
            "tokens": {
                "input": week_stats.0.input_tokens,
                "output": week_stats.0.output_tokens,
                "cache_read": week_stats.0.cache_read_input_tokens,
                "cache_creation": week_stats.0.cache_creation_input_tokens,
                "total": week_stats.0.total()
            },
            "cost": week_stats.1
        },
        "current_month": month_usage.map(|u| {
            serde_json::json!({
                "month": u.month,
                "tokens": {
                    "input": u.tokens.input_tokens,
                    "output": u.tokens.output_tokens,
                    "cache_read": u.tokens.cache_read_input_tokens,
                    "cache_creation": u.tokens.cache_creation_input_tokens,
                    "total": u.tokens.total()
                },
                "cost": u.total_cost,
                "models": u.models_used
            })
        }),
        "all_time": {
            "tokens": {
                "input": stats.total_tokens.input_tokens,
                "output": stats.total_tokens.output_tokens,
                "cache_read": stats.total_tokens.cache_read_input_tokens,
                "cache_creation": stats.total_tokens.cache_creation_input_tokens,
                "total": stats.total_tokens.total()
            },
            "cost": stats.total_cost,
            "sessions": stats.sessions.len()
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    
    Ok(())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}