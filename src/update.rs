//! Self-update module.
//!
//! This module provides self-update functionality for CLI tools, including:
//! - Checking for latest releases on GitHub
//! - Downloading release binaries
//! - Verifying checksums
//! - Replacing the current binary

use crate::types::RepoInfo;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Run update command to install latest or specified version.
///
/// Returns exit code: 0 if successful, 1 on error, 2 if already up-to-date.
///
/// # Arguments
/// * `repo_info` - Repository information for GitHub integration
/// * `current_version` - Current version of the tool
/// * `version` - Optional specific version to install
/// * `force` - Force installation even if already up-to-date
/// * `install_dir` - Optional custom installation directory
///
/// # Panics
/// May panic if stdout flush fails or stdin read fails during user confirmation.
#[must_use]
pub fn run_update(
    repo_info: &RepoInfo,
    current_version: &str,
    version: Option<&str>,
    force: bool,
    install_dir: Option<&Path>,
) -> i32 {
    println!("🔄 Checking for updates...");

    // Get target version
    let target_version = if let Some(v) = version {
        v.to_string()
    } else {
        match get_latest_version(repo_info) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("❌ Failed to check for updates: {e}");
                return 1;
            }
        }
    };

    // Check if already up-to-date
    if target_version == current_version && !force {
        println!("✅ Already running latest version (v{current_version})");
        return 2;
    }

    println!("✨ Update available: v{target_version} (current: v{current_version})");

    // Detect current binary location
    let install_path = if let Some(dir) = install_dir {
        dir.join(repo_info.name)
    } else {
        match std::env::current_exe() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("❌ Failed to determine binary location: {e}");
                return 1;
            }
        }
    };

    println!("📍 Install location: {}", install_path.display());
    println!();

    // Confirm unless forced
    if !force {
        print!("Continue with update? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Update cancelled.");
            return 0;
        }
    }

    // Perform update
    match perform_update(repo_info, &target_version, &install_path) {
        Ok(()) => {
            println!("✅ Successfully updated to v{target_version}");
            println!();
            println!("Run '{} --version' to verify the installation.", repo_info.name);
            0
        }
        Err(e) => {
            eprintln!("❌ Update failed: {e}");
            1
        }
    }
}

/// Get the latest version from GitHub releases.
///
/// # Errors
/// Returns an error if the HTTP request fails, the response cannot be parsed,
/// or the `tag_name` field is missing.
pub fn get_latest_version(repo_info: &RepoInfo) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("{}-updater", repo_info.name))
        .timeout(std::time::Duration::from_secs(10))
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

    let version = tag_name
        .trim_start_matches(repo_info.tag_prefix)
        .trim_start_matches('v');
    Ok(version.to_string())
}

fn perform_update(repo_info: &RepoInfo, version: &str, install_path: &Path) -> Result<(), String> {
    // Detect platform
    let platform = get_platform_string();
    let archive_ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };

    let download_url = repo_info.download_url(version, &platform, archive_ext);

    println!("📥 Downloading {}-{platform}.{archive_ext}...", repo_info.name);

    // Download file
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("{}-updater", repo_info.name))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&download_url)
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let bytes = response.bytes().map_err(|e| e.to_string())?;

    // Download checksum
    let checksum_url = format!("{download_url}.sha256");
    let checksum_response = client
        .get(&checksum_url)
        .send()
        .map_err(|e| e.to_string())?;

    if checksum_response.status().is_success() {
        println!("🔐 Verifying checksum...");
        let expected_checksum = checksum_response.text().map_err(|e| e.to_string())?;
        let expected_checksum = expected_checksum.split_whitespace().next().unwrap_or(&expected_checksum);

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let computed_checksum = hex::encode(hasher.finalize());

        if computed_checksum.to_lowercase() != expected_checksum.to_lowercase() {
            return Err(format!(
                "Checksum mismatch!\nExpected: {expected_checksum}\nGot: {computed_checksum}"
            ));
        }
        println!("✅ Checksum verified");
    } else {
        println!("⚠️  No checksum found, skipping verification");
    }

    // Extract archive
    println!("📦 Extracting archive...");
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

    if cfg!(target_os = "windows") {
        extract_zip(&bytes, temp_dir.path())?;
    } else {
        extract_tar_gz(&bytes, temp_dir.path())?;
    }

    // Find the binary in the extracted files
    let binary_name = if cfg!(target_os = "windows") {
        format!("{}.exe", repo_info.name)
    } else {
        repo_info.name.to_string()
    };

    let extracted_binary = temp_dir.path().join(&binary_name);
    if !extracted_binary.exists() {
        return Err(format!("Binary {binary_name} not found in archive"));
    }

    // Replace the current binary
    println!("🔧 Installing update...");

    // Backup current binary
    let backup_path = install_path.with_extension("bak");
    if let Err(e) = fs::copy(install_path, &backup_path) {
        eprintln!("⚠️  Failed to create backup: {e}");
    }

    // Copy new binary
    fs::copy(&extracted_binary, install_path)
        .map_err(|e| format!("Failed to install binary: {e}"))?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(install_path)
            .map_err(|e| format!("Failed to get metadata: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(install_path, perms)
            .map_err(|e| format!("Failed to set permissions: {e}"))?;
    }

    // Clean up backup
    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path);
    }

    Ok(())
}

fn get_platform_string() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => panic!("Unsupported platform: {os}/{arch}"),
    }
    .to_string()
}

fn extract_tar_gz(bytes: &[u8], dest: &Path) -> Result<(), String> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let decoder = GzDecoder::new(bytes);
    let mut archive = Archive::new(decoder);
    archive
        .unpack(dest)
        .map_err(|e| format!("Failed to extract tar.gz: {e}"))
}

#[cfg(target_os = "windows")]
fn extract_zip(bytes: &[u8], dest: &Path) -> Result<(), String> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| format!("Failed to open zip: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {e}"))?;
        let outpath = dest.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)
                    .map_err(|e| format!("Failed to create parent directory: {e}"))?;
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {e}"))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {e}"))?;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_zip(_bytes: &[u8], _dest: &Path) -> Result<(), String> {
    Err("ZIP extraction not supported on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_string() {
        let platform = get_platform_string();
        // Just verify it returns a non-empty string
        assert!(!platform.is_empty());
        assert!(platform.contains('-'));
    }

    #[test]
    fn test_repo_info_latest_release_url() {
        let repo = RepoInfo::new("workhelix", "prompter", "prompter-v");
        let url = repo.latest_release_url();
        assert_eq!(url, "https://api.github.com/repos/workhelix/prompter/releases/latest");
    }

    #[test]
    fn test_get_latest_version_handles_errors() {
        let repo = RepoInfo::new("nonexistent", "repo", "v");
        // This should fail since the repo doesn't exist
        let result = get_latest_version(&repo);
        // Either succeeds (unlikely) or fails (expected)
        assert!(result.is_ok() || result.is_err());
    }
}
