# tui-split

A terminal UI tool for split-screen layouts built with Rust and ratatui, following TDD principles.

## Features
- Vertical and horizontal split layouts
- PTY (Pseudo-Terminal) support for running real shells (zsh)
- Switch focus between panes
- Test-Driven Development approach

## Architecture
- `lib.rs` - Terminal PTY wrapper with tests
- `main.rs` - TUI application using ratatui

## Development
This project follows t-wada's TDD principles:
1. Red: Write failing tests first
2. Green: Write minimal code to pass tests
3. Refactor: Improve code while keeping tests green

### Running Tests
```
cargo test
```

### Running the Application
```
cargo run
```

### Keyboard shortcuts:
- `q` - Quit
- `v` - Vertical split
- `h` - Horizontal split
- `Tab` - Switch focus between panes

## Current Status
- ✅ Basic PTY implementation with zsh
- ✅ TDD test suite
- ⚠️ Full terminal emulation (ANSI escape sequences) is not yet implemented
- ⚠️ The UI currently shows placeholder text instead of actual terminal output

## Future Work
- Implement proper terminal emulation (vt100/xterm)
- Handle ANSI escape sequences
- Add scrollback buffer
- Support for more shells (bash, fish, etc.)
