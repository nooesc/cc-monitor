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

## How It Works

### Where Does the Data Come From?
Claude Code (the desktop app) automatically logs every interaction you have with Claude AI to local JSONL files on your computer. These files contain:
- **Timestamps** - When you used Claude
- **Token counts** - How many tokens were used (input, output, cache)
- **Model information** - Which Claude model responded
- **Session IDs** - To group related conversations

The logs are stored in:
- `~/.config/claude/projects/` (newer installations)
- `~/.claude/projects/` (older installations)

### What Are Tokens?
Tokens are how AI models measure text. As a rough guide:
- 1 token ≈ 4 characters of English text
- 100 tokens ≈ 75 words
- A typical conversation might use 1,000-10,000 tokens

### Token Types Explained
- **Input tokens**: What you send to Claude (your questions/code)
- **Output tokens**: What Claude sends back (responses/generated code)
- **Cache read tokens**: When Claude reuses previous context (cheaper)
- **Cache creation tokens**: When Claude stores context for reuse

### How Are Costs Calculated?
Each Claude model has different pricing per million tokens:

**Claude 3.5 Sonnet** (most common):
- Input: $3.00 per million tokens
- Output: $15.00 per million tokens
- Cache read: $0.30 per million tokens
- Cache creation: $3.75 per million tokens

**Example**: If you use 10,000 input tokens and get 5,000 output tokens:
- Input cost: 10,000 × ($3.00/1,000,000) = $0.03
- Output cost: 5,000 × ($15.00/1,000,000) = $0.075
- Total: $0.105

### Understanding Sessions
A "session" is one continuous conversation with Claude Code. Each time you start a new chat or open a different project, it creates a new session. Sessions help you track usage per project or task.

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