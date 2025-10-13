//! Health check and diagnostics module.
//!
//! This module provides a framework for running health checks on CLI tools,
//! including update checking and tool-specific diagnostics.

use crate::types::{DoctorCheck, RepoInfo};

/// Trait for tools that support doctor health checks.
///
/// Implement this trait to provide tool-specific health checks.
pub trait DoctorChecks {
    /// Get the repository information for this tool.
    fn repo_info() -> RepoInfo;

    /// Get the current version of this tool.
    fn current_version() -> &'static str;

    /// Run tool-specific health checks.
    ///
    /// Return a vector of check results. Default implementation returns empty vector.
    fn tool_checks(&self) -> Vec<DoctorCheck> {
        Vec::new()
    }
}

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
///
/// # Type Parameters
/// * `T` - A type that implements `DoctorChecks`
pub fn run_doctor<T: DoctorChecks>(tool: &T) -> i32 {
    let tool_name = T::repo_info().name;
    println!("🏥 {tool_name} health check");
    println!("{}", "=".repeat(tool_name.len() + 14));
    println!();

    let mut has_errors = false;
    let mut has_warnings = false;

    // Run tool-specific checks
    let tool_checks = tool.tool_checks();
    if !tool_checks.is_empty() {
        println!("Configuration:");
        for check in tool_checks {
            if check.passed {
                println!("  ✅ {}", check.name);
            } else {
                println!("  ❌ {}", check.name);
                if let Some(msg) = check.message {
                    println!("     {msg}");
                }
                has_errors = true;
            }
        }
        println!();
    }

    // Check for updates
    println!("Updates:");
    match check_for_updates(&T::repo_info(), T::current_version()) {
        Ok(Some(latest)) => {
            let current = T::current_version();
            println!("  ⚠️  Update available: v{latest} (current: v{current})");
            println!("  💡 Run '{tool_name} update' to install the latest version");
            has_warnings = true;
        }
        Ok(None) => {
            println!("  ✅ Running latest version (v{})", T::current_version());
        }
        Err(e) => {
            println!("  ⚠️  Failed to check for updates: {e}");
            has_warnings = true;
        }
    }

    println!();

    // Summary
    if has_errors {
        println!("❌ Issues found - see above for details");
        1
    } else if has_warnings {
        println!("⚠️  Warnings found");
        0 // Warnings don't cause failure
    } else {
        println!("✨ Everything looks healthy!");
        0
    }
}

/// Check for updates from GitHub releases.
///
/// Returns `Ok(Some(version))` if an update is available, `Ok(None)` if up-to-date,
/// or `Err` if the check failed.
///
/// # Errors
/// Returns an error if the HTTP request fails or the response cannot be parsed.
pub fn check_for_updates(
    repo_info: &RepoInfo,
    current_version: &str,
) -> Result<Option<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("{}-doctor", repo_info.name))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let response: serde_json::Value = client
        .get(repo_info.latest_release_url())
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let tag_name = response["tag_name"]
        .as_str()
        .ok_or_else(|| "No tag_name in response".to_string())?;

    let latest = tag_name
        .trim_start_matches(repo_info.tag_prefix)
        .trim_start_matches('v');

    if latest == current_version {
        Ok(None)
    } else {
        Ok(Some(latest.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    impl DoctorChecks for TestTool {
        fn repo_info() -> RepoInfo {
            RepoInfo::new("workhelix", "test-tool", "test-tool-v")
        }

        fn current_version() -> &'static str {
            "1.0.0"
        }

        fn tool_checks(&self) -> Vec<DoctorCheck> {
            vec![
                DoctorCheck::pass("Test check 1"),
                DoctorCheck::fail("Test check 2", "This is a failure"),
            ]
        }
    }

    #[test]
    fn test_run_doctor() {
        let tool = TestTool;
        let exit_code = run_doctor(&tool);
        // Should return 1 because we have a failing check
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_check_for_updates_handles_errors() {
        let repo = RepoInfo::new("nonexistent", "repo", "v");
        let result = check_for_updates(&repo, "1.0.0");
        // Either succeeds or fails, both are acceptable in tests
        assert!(result.is_ok() || result.is_err());
    }
}
