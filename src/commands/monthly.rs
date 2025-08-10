use anyhow::Result;
use crate::data_loader::DataLoader;

pub fn show_monthly(json: bool) -> Result<()> {
    let loader = DataLoader::new()?;
    let stats = loader.load_all_usage()?;
    
    if json {
        let output = serde_json::json!({
            "monthly": stats.monthly.iter().map(|m| {
                serde_json::json!({
                    "month": m.month,
                    "tokens": {
                        "input": m.tokens.input_tokens,
                        "output": m.tokens.output_tokens,
                        "total": m.tokens.total()
                    },
                    "cost": m.total_cost,
                    "models": m.models_used
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("📈 Monthly Usage Report\n");
        println!("{:<10} {:>15} {:>15} {:>10}", "Month", "Input Tokens", "Output Tokens", "Cost");
        println!("{}", "─".repeat(53));
        
        for entry in &stats.monthly {
            println!("{:<10} {:>15} {:>15} ${:>9.2}",
                entry.month,
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