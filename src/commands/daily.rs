use anyhow::Result;
use crate::data_loader::DataLoader;

pub fn show_daily(json: bool, days: usize) -> Result<()> {
    let loader = DataLoader::new()?;
    let stats = loader.load_all_usage()?;
    
    let daily_entries: Vec<_> = stats.daily.iter()
        .rev()
        .take(days)
        .collect();
    
    if json {
        let output = serde_json::json!({
            "daily": daily_entries.iter().map(|d| {
                serde_json::json!({
                    "date": d.date.to_string(),
                    "tokens": {
                        "input": d.tokens.input_tokens,
                        "output": d.tokens.output_tokens,
                        "total": d.tokens.total()
                    },
                    "cost": d.total_cost,
                    "models": d.models_used
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("📅 Daily Usage Report (Last {} days)\n", days);
        println!("{:<12} {:>15} {:>15} {:>10}", "Date", "Input Tokens", "Output Tokens", "Cost");
        println!("{}", "─".repeat(55));
        
        for entry in daily_entries.iter().rev() {
            println!("{:<12} {:>15} {:>15} ${:>9.2}",
                entry.date.format("%Y-%m-%d"),
                format_number(entry.tokens.input_tokens),
                format_number(entry.tokens.output_tokens),
                entry.total_cost);
        }
    }
    
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