use anyhow::Result;
use crate::data_loader::DataLoader;

pub fn show_sessions(json: bool, limit: usize) -> Result<()> {
    let loader = DataLoader::new()?;
    let stats = loader.load_all_usage()?;
    
    let sessions: Vec<_> = stats.sessions.iter()
        .take(limit)
        .collect();
    
    if json {
        let output = serde_json::json!({
            "sessions": sessions.iter().map(|s| {
                serde_json::json!({
                    "session_id": s.session_id,
                    "project_path": s.project_path,
                    "last_activity": s.last_activity.to_rfc3339(),
                    "tokens": {
                        "input": s.tokens.input_tokens,
                        "output": s.tokens.output_tokens,
                        "total": s.tokens.total()
                    },
                    "cost": s.total_cost,
                    "models": s.models_used
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("🔍 Session Usage Report (Last {} sessions)\n", limit);
        println!("{:<20} {:>15} {:>10} {:<30}", "Last Activity", "Total Tokens", "Cost", "Project");
        println!("{}", "─".repeat(78));
        
        for session in sessions {
            let project = if session.project_path.len() > 28 {
                format!("{}...", &session.project_path[..25])
            } else {
                session.project_path.clone()
            };
            
            println!("{:<20} {:>15} ${:>9.2} {:<30}",
                session.last_activity.format("%Y-%m-%d %H:%M"),
                format_number(session.tokens.total()),
                session.total_cost,
                project);
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