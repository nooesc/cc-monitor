# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CC-Monitor is a minimal Rust CLI tool for monitoring Claude Code usage. It provides two simple commands:
- **Dashboard** (default): Interactive TUI for browsing usage data
- **Statusline**: Compact output for Claude Code hooks integration

## Key Commands

### Building and Running
```bash
# Build the project
cargo build

# Run dashboard (default)
cargo run

# Run statusline
cargo run -- statusline

# Build release version (optimized, fast)
cargo build --release

# Install locally
cargo install --path .
```

### Testing
```bash
# Test dashboard
cargo run

# Test statusline
cargo run -- statusline
echo '{"model": "claude-3-5-sonnet-20241022"}' | cargo run -- statusline --stdin
```

## Architecture

### Simplified Structure
- **Two commands only**: Dashboard (default) and Statusline
- **Automatic data discovery**: Finds Claude logs in standard locations
- **Real-time monitoring**: Dashboard refreshes every 5 seconds
- **Hook integration**: Statusline designed for Claude Code settings.json

### Core Data Flow
1. **Data Discovery**: Finds JSONL files in `~/.config/claude/projects/` or `~/.claude/projects/`
2. **Session Parsing**: Extracts session IDs from file paths
3. **Token Aggregation**: Groups by session with cache deduplication
4. **Cost Calculation**: Applies model-specific pricing
5. **Display**: Either TUI dashboard or compact statusline

### Module Organization
```
src/
├── main.rs              # Entry point, routes to dashboard or statusline
├── cli/mod.rs           # Simple CLI with Option<Commands>
├── data_loader.rs       # Core data loading logic
├── models/              # Data structures
├── commands/
│   └── statusline.rs    # Statusline implementation
└── tui/                 # Dashboard implementation
```

### Critical Logic

#### Session Chain Detection (data_loader.rs:712-730)
Sessions within 10 minutes in the same project are "resumed" - prevents double-counting cache tokens.

#### Default Command (main.rs:29)
Dashboard runs by default when no command specified: `Some(Commands::Dashboard) | None =>`

## Common Development Tasks

### Updating Model Pricing
Edit `src/models/pricing.rs` - the `from_model()` method:
```rust
"claude-new-model" => ModelPricing {
    input_per_million: 3.00,
    output_per_million: 15.00,
    cache_creation_per_million: Some(3.75),
    cache_read_per_million: Some(0.30),
}
```

### Modifying Statusline Format
Edit `src/commands/statusline.rs` - the output formatting logic for hook integration.

### Dashboard Improvements
Edit `src/tui/dashboard.rs` for UI changes, `src/tui/app.rs` for state management.

## Design Philosophy

This tool is intentionally minimal:
- **No complex commands**: Just dashboard and statusline
- **No configuration files**: Works out of the box
- **Fast startup**: Optimized for quick checks
- **Clean output**: Designed for terminal/hook usage