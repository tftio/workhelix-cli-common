# workhelix-cli-common

Common functionality for Workhelix Rust CLI tools.

## Overview

This library provides shared functionality for CLI tools including:

- **Shell completion generation** - Generate completions for bash, zsh, fish, elvish, PowerShell
- **Health check framework** - Extensible doctor command with tool-specific checks
- **License display** - Standardized license information for MIT, Apache-2.0, CC0
- **Terminal output utilities** - TTY-aware colored output and formatting

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
workhelix-cli-common = "0.1"
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
```

Or add via command line:

```bash
cargo add workhelix-cli-common
```

### Using a Local Development Version

For local development:

```toml
[dependencies]
workhelix-cli-common = { path = "../workhelix-cli-common" }
```

## Usage

## Example

```rust
use workhelix_cli_common::{
    RepoInfo, DoctorChecks, DoctorCheck,
    completions, doctor, license, LicenseType,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Version,
    License,
    Completions { shell: clap_complete::Shell },
    Doctor,
}

struct MyTool;

impl DoctorChecks for MyTool {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("myorg", "mytool")
    }

    fn current_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn tool_checks(&self) -> Vec<DoctorCheck> {
        vec![
            DoctorCheck::file_exists("~/.config/mytool/config.toml"),
        ]
    }
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Version => {
            println!("mytool {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Commands::License => {
            println!("{}", license::display_license("mytool", LicenseType::MIT));
            0
        }
        Commands::Completions { shell } => {
            completions::generate_completions::<Cli>(shell);
            0
        }
        Commands::Doctor => {
            doctor::run_doctor(&MyTool)
        }
    };

    std::process::exit(exit_code);
}
```

## Features

### Completions

Generate shell completions with installation instructions:

```rust
completions::generate_completions::<YourCli>(Shell::Bash);
```

### Doctor

Health checks with extensible framework:

```rust
impl DoctorChecks for MyTool {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("owner", "repo")
    }

    fn current_version() -> &'static str {
        "1.0.0"
    }

    fn tool_checks(&self) -> Vec<DoctorCheck> {
        vec![
            DoctorCheck::file_exists("/path/to/config"),
            DoctorCheck::dir_exists("/path/to/data"),
        ]
    }
}

let tool = MyTool;
doctor::run_doctor(&tool);
```

### License

Display license information:

```rust
use workhelix_cli_common::{license, LicenseType};

println!("{}", license::display_license("mytool", LicenseType::MIT));
```

### Output Utilities

TTY-aware colored output:

```rust
use workhelix_cli_common::output;

println!("{}", output::success("Operation completed"));
println!("{}", output::error("Operation failed"));
println!("{}", output::warning("Warning message"));
println!("{}", output::info("Information"));
```

## Links

- [crates.io](https://crates.io/crates/workhelix-cli-common)
- [Documentation](https://docs.rs/workhelix-cli-common)
- [Repository](https://github.com/tftio/workhelix-cli-common)

## Development

See [RELEASE.md](RELEASE.md) for information about the release process.

## License

MIT License - See LICENSE file for details.
