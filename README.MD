# CC-Monitor

A simple, fast Rust CLI to monitor Claude Code usage.

## Installation

```bash
cargo install --path .
```

## Usage

### Interactive Dashboard (default)
```bash
cc-monitor
# or explicitly:
cc-monitor dashboard
```
Navigate with Tab, ↑↓ arrows, and press 'q' to quit.

### Statusline (for Claude hooks)
Compact one-line output perfect for status bars or Claude Code hooks:
```bash
cc-monitor statusline          # Basic statusline
cc-monitor statusline --stdin  # Read model info from stdin (for hooks)
```

Output format: `🤖 Model | 💰 $X session / $Y today / $Z block | 🔥 $W/hr`

To use with Claude Code hooks, add to `~/.claude/settings.json`:
```json
{
  "hooks": {
    "on_prompt_submit": "cc-monitor statusline --stdin"
  }
}
```

## How It Works

### Where Does the Data Come From?
Claude Code automatically logs every interaction to local JSONL files on your computer. These files contain:
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

### Token Types
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

## License

MIT