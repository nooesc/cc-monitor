use crate::data_loader::DataLoader;
use anyhow::Result;
use chrono::{DateTime, Duration, Local, Utc};
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub session_id: String,
    #[allow(dead_code)]
    pub transcript_path: String,
    #[allow(dead_code)]
    pub cwd: String,
    pub model: ModelInfo,
    #[allow(dead_code)]
    pub workspace: WorkspaceInfo,
    #[allow(dead_code)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct WorkspaceInfo {
    pub current_dir: String,
    pub project_dir: String,
}

pub fn show_statusline(read_stdin: bool) -> Result<()> {
    let loader = DataLoader::new()?;
    let stats = loader.load_all_usage()?;

    // Get current model and session ID from stdin if available
    let (model_name, session_id) = if read_stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        if let Ok(hook_data) = serde_json::from_str::<HookInput>(&buffer) {
            (Some(hook_data.model.display_name), Some(hook_data.session_id))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Calculate today's stats
    let today = Local::now().date_naive();
    let today_usage = stats.daily.iter().find(|d| d.date == today);

    let today_cost = today_usage.map(|u| u.total_cost).unwrap_or(0.0);

    // Calculate CURRENT session cost (not all recent sessions!)
    let session_cost = if let Some(sid) = &session_id {
        stats.sessions.iter()
            .find(|s| s.session_id == *sid)
            .map(|s| s.total_cost)
            .unwrap_or(0.0)  // New session shows $0.00
    } else {
        // No session ID provided - show 0.0 instead of guessing
        0.0
    };

    // Calculate burn rate (tokens per hour over last 3 hours)
    let three_hours_ago = Utc::now() - Duration::hours(3);
    let recent_sessions_3h: Vec<_> = stats
        .sessions
        .iter()
        .filter(|s| s.last_activity > three_hours_ago)
        .collect();

    let _tokens_3h: u64 = recent_sessions_3h.iter().map(|s| s.tokens.total()).sum();

    let cost_3h: f64 = recent_sessions_3h.iter().map(|s| s.total_cost).sum();

    let burn_rate = if !recent_sessions_3h.is_empty() {
        cost_3h / 3.0 // Cost per hour
    } else {
        0.0
    };

    // Determine burn rate color/emoji
    let (burn_emoji, burn_color) = match burn_rate {
        x if x > 10.0 => ("🔥", "\x1b[91m"), // Red for high burn
        x if x > 5.0 => ("🔥", "\x1b[93m"),  // Yellow for medium burn
        x if x > 0.0 => ("🔥", "\x1b[92m"),  // Green for low burn
        _ => ("💤", "\x1b[90m"),             // Gray for no activity
    };

    // Get current 5-hour block stats
    let now = Utc::now();
    let hours_since_epoch = now.timestamp() / 3600;
    let block_start_hours = (hours_since_epoch / 5) * 5;
    let block_start = DateTime::<Utc>::from_timestamp(block_start_hours * 3600, 0).unwrap_or(now);

    let block_sessions: Vec<_> = stats
        .sessions
        .iter()
        .filter(|s| s.last_activity >= block_start)
        .collect();

    let block_cost: f64 = block_sessions.iter().map(|s| s.total_cost).sum();

    let block_tokens: u64 = block_sessions.iter().map(|s| s.tokens.total()).sum();

    // Calculate time remaining in block
    let block_end = block_start + Duration::hours(5);
    let remaining = block_end - now;
    let hours_remaining = remaining.num_hours();
    let minutes_remaining = remaining.num_minutes() % 60;

    // Format the statusline
    print!(
        "💰 ${:.2} session / ${:.2} today / ",
        session_cost, today_cost
    );
    print!(
        "${:.2} block ({:02}:{:02} left) | ",
        block_cost, hours_remaining, minutes_remaining
    );
    print!("{}{} ${:.2}/hr\x1b[0m", burn_color, burn_emoji, burn_rate);

    println!(); // End line

    Ok(())
}

#[allow(dead_code)]
pub fn show_statusline_json(read_stdin: bool) -> Result<()> {
    let loader = DataLoader::new()?;
    let stats = loader.load_all_usage()?;

    // Get hook input if available
    let hook_data = if read_stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        serde_json::from_str::<HookInput>(&buffer).ok()
    } else {
        None
    };

    // Calculate all the same stats as above
    let today = Local::now().date_naive();
    let today_usage = stats.daily.iter().find(|d| d.date == today);

    let today_cost = today_usage.map(|u| u.total_cost).unwrap_or(0.0);
    let today_tokens = today_usage.map(|u| u.tokens.total()).unwrap_or(0);

    // Calculate CURRENT session cost (not all recent sessions!)
    let session_cost = if let Some(ref hd) = hook_data {
        stats.sessions.iter()
            .find(|s| s.session_id == hd.session_id)
            .map(|s| s.total_cost)
            .unwrap_or(0.0)  // New session shows $0.00
    } else {
        // No hook data - show 0.0 instead of guessing
        0.0
    };

    let three_hours_ago = Utc::now() - Duration::hours(3);
    let recent_sessions_3h: Vec<_> = stats
        .sessions
        .iter()
        .filter(|s| s.last_activity > three_hours_ago)
        .collect();

    let cost_3h: f64 = recent_sessions_3h.iter().map(|s| s.total_cost).sum();

    let burn_rate = if !recent_sessions_3h.is_empty() {
        cost_3h / 3.0
    } else {
        0.0
    };

    // Block calculations
    let now = Utc::now();
    let hours_since_epoch = now.timestamp() / 3600;
    let block_start_hours = (hours_since_epoch / 5) * 5;
    let block_start = DateTime::<Utc>::from_timestamp(block_start_hours * 3600, 0).unwrap_or(now);

    let block_sessions: Vec<_> = stats
        .sessions
        .iter()
        .filter(|s| s.last_activity >= block_start)
        .collect();

    let block_cost: f64 = block_sessions.iter().map(|s| s.total_cost).sum();

    let block_tokens: u64 = block_sessions.iter().map(|s| s.tokens.total()).sum();

    let block_end = block_start + Duration::hours(5);
    let remaining_minutes = (block_end - now).num_minutes();

    let output = serde_json::json!({
        "model": hook_data.as_ref().map(|h| &h.model.display_name),
        "session": {
            "cost": session_cost,
            "description": "Current session"
        },
        "today": {
            "cost": today_cost,
            "tokens": today_tokens,
            "date": today.to_string()
        },
        "block": {
            "cost": block_cost,
            "tokens": block_tokens,
            "start": block_start.to_rfc3339(),
            "end": block_end.to_rfc3339(),
            "remaining_minutes": remaining_minutes
        },
        "burn_rate": {
            "cost_per_hour": burn_rate,
            "period_hours": 3,
            "status": if burn_rate > 10.0 { "high" }
                     else if burn_rate > 5.0 { "medium" }
                     else if burn_rate > 0.0 { "low" }
                     else { "idle" }
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
