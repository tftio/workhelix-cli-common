# workhelix-cli-common

Common functionality for Workhelix Rust CLI tools.

## Overview

This library provides shared functionality for CLI tools including:

- **Shell completion generation** - Generate completions for bash, zsh, fish, elvish, PowerShell
- **Self-update mechanism** - Download and install updates from GitHub releases with checksum verification
- **Health check framework** - Extensible doctor command with tool-specific checks
- **License display** - Standardized license information for MIT, Apache-2.0, CC0
- **Terminal output utilities** - TTY-aware colored output and formatting

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
workhelix-cli-common = { path = "../workhelix-cli-common" }
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
```

## Example

```rust
use workhelix_cli_common::{
    RepoInfo, DoctorChecks, DoctorCheck,
    completions, doctor, license, update, LicenseType,
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
    Update {
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        install_dir: Option<std::path::PathBuf>,
    },
}

struct MyTool;

impl DoctorChecks for MyTool {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("myorg", "mytool", "mytool-v")
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
        Commands::Update { version, force, install_dir } => {
            update::run_update(
                &MyTool::repo_info(),
                MyTool::current_version(),
                version.as_deref(),
                force,
                install_dir.as_deref(),
            )
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

### Update

Self-update with GitHub releases integration:

```rust
let repo = RepoInfo::new("owner", "repo", "repo-v");
update::run_update(&repo, "1.0.0", None, false, None);
```

### Doctor

Health checks with extensible framework:

```rust
impl DoctorChecks for MyTool {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("owner", "repo", "v")
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

## License

MIT License - See LICENSE file for details.
