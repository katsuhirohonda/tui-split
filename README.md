# tui-split

A simple terminal UI tool for split-screen layouts built with Rust and ratatui. Each pane can run different shell commands and display their output.

## Features
- Vertical and horizontal split layouts
- Run shell commands in each pane
- Auto-refresh command output every 2 seconds
- Switch focus between panes
- Scroll through command output
- Change commands on the fly

## Usage
```
cargo run
```

### Keyboard shortcuts:
- `q` - Quit
- `v` - Vertical split
- `h` - Horizontal split
- `Tab` - Switch focus between panes
- `↑/↓` - Scroll up/down in focused pane
- `1` - Change pane 1 to disk usage
- `2` - Change pane 2 to network info
- `3` - Change pane 1 to memory info
- `4` - Change pane 2 to CPU info

## Default Commands
- Pane 1: System information (date, uname, uptime)
- Pane 2: Process list (ps aux)
