# CC-Monitor

A simple, fast Rust tool to monitor Claude Code usage with an interactive dashboard.

## Features

- 📊 **Status Command**: Quick overview of your Claude usage
- 📈 **Daily/Monthly Reports**: Detailed breakdowns by time period
- 🔍 **Session Tracking**: Monitor individual Claude sessions
- 🎨 **Interactive Dashboard**: Real-time TUI with ratatui
- 💰 **Cost Tracking**: Automatic cost calculation based on model pricing
- 🚀 **Fast**: Written in Rust for blazing fast performance

## Installation

```bash
cargo install --path .
```

## Usage

### Quick Status
```bash
cc-monitor status
cc-monitor status --detailed  # More details
cc-monitor status --json      # JSON output
```

### Interactive Dashboard
```bash
cc-monitor dashboard
```
Navigate with Tab, ↑↓ arrows, and press 'q' to quit.

### Reports
```bash
cc-monitor daily --days 7     # Last 7 days
cc-monitor monthly            # Monthly breakdown
cc-monitor sessions --limit 10 # Recent sessions
```

### JSON Output
All commands support `--json` flag for programmatic access:
```bash
cc-monitor status --json | jq '.today.cost'
```

## Configuration

Set custom Claude data directories:
```bash
export CLAUDE_CONFIG_DIR=/path/to/claude
# Or multiple directories:
export CLAUDE_CONFIG_DIR=/path1,/path2
```

By default, searches both:
- `~/.config/claude/projects/` (new location)
- `~/.claude/projects/` (legacy location)

## Supported Models

Currently tracks pricing for:
- Claude 3.5 Sonnet
- Claude 3.5 Haiku  
- Claude 3 Opus
- Claude 4 models (when available)

## License

MIT