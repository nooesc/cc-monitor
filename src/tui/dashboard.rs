use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline, Tabs},
    Frame, Terminal,
};
use std::io;

use crate::tui::app::{App, Tab};

pub fn run_dashboard(app: App) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = app;
    
    // Main loop
    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;
        
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.quit(),
                KeyCode::Tab => app.next_tab(),
                KeyCode::BackTab => app.previous_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                _ => {}
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}

fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());
    
    // Draw header with tabs
    draw_header(f, app, chunks[0]);
    
    // Draw main content based on selected tab
    match app.selected_tab {
        Tab::Overview => draw_overview(f, app, chunks[1]),
        Tab::Daily => draw_daily(f, app, chunks[1]),
        Tab::Sessions => draw_sessions(f, app, chunks[1]),
        Tab::Monthly => draw_monthly(f, app, chunks[1]),
    }
    
    // Draw footer
    draw_footer(f, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Overview", "Daily", "Sessions", "Monthly"];
    let selected = match app.selected_tab {
        Tab::Overview => 0,
        Tab::Daily => 1,
        Tab::Sessions => 2,
        Tab::Monthly => 3,
    };
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Claude Code Monitor "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    f.render_widget(tabs, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" to switch tabs, "),
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(" to navigate, "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(" to quit"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    
    f.render_widget(footer, area);
}

fn draw_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Stats cards
            Constraint::Min(0),     // Chart
        ])
        .split(area);
    
    // Draw stats cards
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);
    
    // Today's stats
    let today_stats = app.get_today_stats();
    let today_text = if let Some(stats) = today_stats {
        vec![
            Line::from(Span::styled("Today", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(format!("Tokens: {}", format_number(stats.tokens.total()))),
            Line::from(format!("Cost: ${:.2}", stats.total_cost)),
        ]
    } else {
        vec![
            Line::from(Span::styled("Today", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from("No usage yet"),
        ]
    };
    
    let today_widget = Paragraph::new(today_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(today_widget, stats_chunks[0]);
    
    // Week stats
    let (week_tokens, week_cost) = app.get_week_stats();
    let week_text = vec![
        Line::from(Span::styled("Last 7 Days", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(format!("Tokens: {}", format_number(week_tokens.total()))),
        Line::from(format!("Cost: ${:.2}", week_cost)),
    ];
    
    let week_widget = Paragraph::new(week_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(week_widget, stats_chunks[1]);
    
    // Month stats
    let month_stats = app.get_month_stats();
    let month_text = if let Some(stats) = month_stats {
        vec![
            Line::from(Span::styled("This Month", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(format!("Tokens: {}", format_number(stats.tokens.total()))),
            Line::from(format!("Cost: ${:.2}", stats.total_cost)),
        ]
    } else {
        vec![
            Line::from(Span::styled("This Month", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from("No usage yet"),
        ]
    };
    
    let month_widget = Paragraph::new(month_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(month_widget, stats_chunks[2]);
    
    // All-time stats
    let total_text = vec![
        Line::from(Span::styled("All Time", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(format!("Tokens: {}", format_number(app.stats.total_tokens.total()))),
        Line::from(format!("Cost: ${:.2}", app.stats.total_cost)),
        Line::from(format!("Sessions: {}", app.stats.sessions.len())),
    ];
    
    let total_widget = Paragraph::new(total_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(total_widget, stats_chunks[3]);
    
    // Draw sparkline chart
    let daily_costs: Vec<u64> = app.stats.daily.iter()
        .rev()
        .take(30)
        .map(|d| (d.total_cost * 100.0) as u64)
        .rev()
        .collect();
    
    if !daily_costs.is_empty() {
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(" Daily Usage (Last 30 Days) "))
            .data(&daily_costs)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(sparkline, chunks[1]);
    }
}

fn draw_daily(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.stats.daily.iter()
        .rev()
        .enumerate()
        .map(|(i, d)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<12}", d.date.format("%Y-%m-%d")), style),
                Span::raw("  "),
                Span::styled(format!("{:>10} tokens", format_number(d.tokens.total())), style),
                Span::raw("  "),
                Span::styled(format!("${:>8.2}", d.total_cost), style),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Daily Usage "));
    
    f.render_widget(list, area);
}

fn draw_sessions(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.stats.sessions.iter()
        .take(20)
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let project = if s.project_path.len() > 40 {
                format!("{}...", &s.project_path[..37])
            } else {
                s.project_path.clone()
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<20}", s.last_activity.format("%Y-%m-%d %H:%M")), style),
                Span::raw("  "),
                Span::styled(format!("{:>10} tokens", format_number(s.tokens.total())), style),
                Span::raw("  "),
                Span::styled(format!("${:>8.2}", s.total_cost), style),
                Span::raw("  "),
                Span::styled(project, style),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Recent Sessions "));
    
    f.render_widget(list, area);
}

fn draw_monthly(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.stats.monthly.iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<10}", m.month), style),
                Span::raw("  "),
                Span::styled(format!("{:>12} tokens", format_number(m.tokens.total())), style),
                Span::raw("  "),
                Span::styled(format!("${:>10.2}", m.total_cost), style),
                Span::raw("  "),
                Span::styled(format!("{} models", m.models_used.len()), style),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Monthly Usage "));
    
    f.render_widget(list, area);
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