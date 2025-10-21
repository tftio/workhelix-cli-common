# CLAUDE.md

This file contains project documentation specifically for Claude AI assistant interactions.

## Project Overview

**workhelix-cli-common** is a Rust library providing shared functionality for CLI tools in the Workhelix ecosystem. It offers a standardized set of utilities for shell completions, self-updates, health checks, license display, and terminal output formatting.

- **Language**: Rust (Edition 2024, MSRV 1.85.0)
- **License**: MIT
- **Repository**: https://github.com/tftio/workhelix-cli-common
- **Current Version**: 0.3.1

## Project Structure

```
workhelix-cli-common/
├── Cargo.toml              # Package manifest with dependencies
├── README.md               # User-facing documentation
├── RELEASE.md              # Release process documentation
├── LICENSE                 # MIT license text
├── VERSION                 # Current version file
├── justfile               # Just command runner recipes
├── hooks.toml             # Pre-commit hooks configuration
├── src/
│   ├── lib.rs             # Main library entry point with re-exports
│   ├── types.rs           # Core data structures (RepoInfo, DoctorCheck)
│   ├── completions.rs     # Shell completion generation
│   ├── doctor.rs          # Health check framework
│   ├── license.rs         # License display utilities
│   ├── output.rs          # Terminal output formatting
│   └── update.rs          # Self-update mechanism
└── target/                # Build artifacts directory
```

## Core Modules

### 1. **types.rs** - Core Data Structures
- `RepoInfo`: Repository metadata (owner, name, tag_prefix)
- `DoctorCheck`: Health check results with pass/fail status

### 2. **completions.rs** - Shell Completion Generation
- Generates completions for bash, zsh, fish, elvish, PowerShell
- Provides installation instructions for each shell
- Uses clap_complete for completion generation

### 3. **doctor.rs** - Health Check Framework
- Trait `DoctorChecks` for implementing tool-specific health checks
- Built-in checks for updates, files, directories
- Colored output for check results
- Returns appropriate exit codes

### 4. **license.rs** - License Display
- Support for MIT, Apache-2.0, CC0-1.0 licenses
- Template-based license text generation
- Consistent formatting across tools

### 5. **output.rs** - Terminal Output Utilities
- TTY-aware colored output (success, error, warning, info)
- Conditional coloring based on terminal capabilities
- Uses `colored` and `is-terminal` crates

### 6. **update.rs** - Self-Update Mechanism
- GitHub releases integration
- Checksum verification for downloads
- Cross-platform binary installation
- Progress feedback and error handling

## Key Dependencies

```toml
clap = "4.5"              # Command-line argument parsing
clap_complete = "4.5"     # Shell completion generation
colored = "3.0"           # Terminal color output
is-terminal = "0.4"       # TTY detection
reqwest = "0.12"          # HTTP client for updates
serde_json = "1.0"        # JSON serialization
cargo-edit = "0.13.7"     # Cargo manifest editing
```

## Development Guidelines

### Code Quality Standards
- Maximum clippy lint levels (all, pedantic, nursery, cargo)
- Missing documentation denied (`missing_docs = "deny"`)
- Unsafe code warned (`unsafe_code = "warn"`)
- Edition 2024 features enabled

### Architecture Principles
1. **Modular Design**: Each module has a single responsibility
2. **Trait-based Extensibility**: `DoctorChecks` trait allows tool customization
3. **Error Handling**: Consistent exit codes and error reporting
4. **Cross-platform Support**: Windows, macOS, Linux compatibility
5. **TTY Awareness**: Conditional formatting based on terminal capabilities

### Testing Strategy
- Unit tests in `lib.rs` for core functionality
- Integration tests for each module's public API
- Cross-platform testing for file operations

## Common Usage Patterns

### Implementing a CLI Tool
1. Define CLI structure with clap derive macros
2. Implement `DoctorChecks` trait for tool-specific health checks
3. Use provided utilities for completions, updates, license display
4. Handle exit codes consistently across commands

### Adding New Health Checks
- Extend `DoctorCheck` with new check types
- Add corresponding logic in `doctor.rs`
- Maintain consistent output formatting

### Integration Notes
- Tools should use `env!("CARGO_PKG_VERSION")` for version info
- Repository info follows pattern: owner/repo with tag prefix
- Exit codes: 0 = success, 1 = failure, 2 = partial success

## Recent Changes (v0.3.1)
- Rust Edition 2024 migration
- Enhanced linting configuration
- Improved error handling in update mechanism
- Better cross-platform support

This documentation should be updated when significant architectural changes or new modules are added to the project.