# Release Process

This document describes the release process for `workhelix-cli-common`.

## Overview

The release process is automated using:
- **Local workflow**: `justfile` recipes for version management
- **GitHub Actions**: Automated CI/CD and publishing to crates.io
- **Trusted Publishing**: Secure OIDC-based authentication (no manual tokens)

## Prerequisites

### One-Time Setup

#### 1. Install Required Tools

```bash
# Install versioneer for version management
cargo install versioneer

# Install peter-hook for git hooks
cargo install peter-hook

# Install audit and deny for security checks
cargo install cargo-audit cargo-deny
```

#### 2. Set Up crates.io Account

1. Visit [crates.io](https://crates.io) and log in with GitHub
2. Verify your email address at [Account Settings](https://crates.io/settings/profile)
3. For first release only: Create API token at [https://crates.io/me](https://crates.io/me)
4. For first release only: Run `cargo login` and paste the token

#### 3. Configure Trusted Publishing (After First Release)

After publishing the first version manually:

1. Go to your crate's settings on crates.io
2. Navigate to the "Publishing" section
3. Add your GitHub repository: `tftio/workhelix-cli-common`
4. Configure the workflow name: `publish.yml`
5. Save the configuration

This allows GitHub Actions to publish automatically without storing tokens.

## Release Workflow

### Standard Release Process

1. **Ensure clean working directory**:
   ```bash
   git status
   # Should show no uncommitted changes
   ```

2. **Run the release command**:
   ```bash
   # For patch version (0.1.3 → 0.1.4)
   just release patch

   # For minor version (0.1.3 → 0.2.0)
   just release minor

   # For major version (0.1.3 → 1.0.0)
   just release major
   ```

3. **What happens automatically**:
   - ✅ Prerequisites validation (clean working directory, on main branch, synced with remote)
   - ✅ Quality gates (tests, clippy, security audit, dependency compliance)
   - ✅ Package validation (`cargo publish --dry-run`)
   - ✅ Version bump in `Cargo.toml` and `VERSION`
   - ✅ Git commit with message `chore: bump version to X.Y.Z`
   - ✅ Git tag creation `vX.Y.Z`
   - ✅ Interactive confirmation prompt
   - ✅ Push to GitHub (commit + tag)

4. **GitHub Actions takes over**:
   - 🔄 Runs full CI test suite on all platforms
   - 🔄 Validates tag matches `Cargo.toml` version
   - 🔄 Publishes to crates.io via Trusted Publishing
   - 🔄 Creates GitHub Release with auto-generated notes

5. **Monitor the release**:
   - Check [GitHub Actions](https://github.com/tftio/workhelix-cli-common/actions)
   - Verify on [crates.io](https://crates.io/crates/workhelix-cli-common)
   - Check [GitHub Releases](https://github.com/tftio/workhelix-cli-common/releases)

### First Release Only

The very first release must be published manually:

```bash
# After running `just release patch` and pushing the tag
cargo publish
```

After this, configure Trusted Publishing (see above) so future releases are automatic.

## Quality Gates

All releases must pass:

- **Format check**: `cargo +nightly fmt --check`
- **Clippy**: `cargo clippy --all-targets -- -D warnings`
- **Tests**: `cargo test --all --verbose`
- **Security audit**: `cargo audit`
- **Dependency compliance**: `cargo deny check`
- **Package validation**: `cargo publish --dry-run`

## Continuous Integration

### On Every Push/PR

The CI workflow (`.github/workflows/ci.yml`) runs:
- Format check
- Clippy linting
- Tests on Linux, macOS, and Windows
- Security audit
- Dependency compliance check

### On Tag Push

The publish workflow (`.github/workflows/publish.yml`) runs:
- Version validation
- Full test suite
- Package build verification
- Publishing to crates.io
- GitHub Release creation

## Version Management

This project uses semantic versioning (SemVer):

- **Patch** (0.1.3 → 0.1.4): Bug fixes, minor changes
- **Minor** (0.1.3 → 0.2.0): New features, backward compatible
- **Major** (0.1.3 → 1.0.0): Breaking changes

Version synchronization:
- `VERSION` file: Source of truth
- `Cargo.toml`: Automatically synced by versioneer
- Git tags: Format `vX.Y.Z`

## Troubleshooting

### Release Failed: "Working directory is not clean"

Commit or stash your changes:
```bash
git status
git add .
git commit -m "your message"
```

### Release Failed: "Not up-to-date with origin/main"

Pull the latest changes:
```bash
git pull origin main
```

### GitHub Actions Publishing Failed

Check if Trusted Publishing is configured:
1. Go to crates.io → Your Crate → Settings → Publishing
2. Verify GitHub repository is linked
3. Verify workflow name matches: `publish.yml`

### Need to Publish Without GitHub Actions

```bash
# Get an API token from https://crates.io/me
cargo login

# Publish manually
cargo publish
```

## Manual Operations

### Check Current Version

```bash
just version-show
```

### Validate Package Without Release

```bash
cargo package --locked
cargo publish --dry-run
```

### Manually Create Tag

```bash
# Only if not using the release workflow
git tag -a v0.1.4 -m "Release v0.1.4"
git push --tags
```

## Best Practices

1. **Always use the `just release` command** - It ensures all quality gates pass
2. **Never skip quality checks** - They prevent publishing broken versions
3. **Test locally before releasing** - Run `just dev` or `just ci`
4. **Review the CHANGELOG** - Keep users informed of changes
5. **Use semantic versioning** - Makes version expectations clear
6. **Monitor GitHub Actions** - Ensure automated publishing succeeds

## Architecture

```
Developer          GitHub              crates.io
    |                 |                    |
    |  just release   |                    |
    |---------------->|                    |
    |                 |                    |
    |  git push tags  |                    |
    |---------------->|                    |
    |                 |                    |
    |                 | GitHub Actions     |
    |                 | (CI + Publish)     |
    |                 |------------------->|
    |                 |                    |
    |                 | Trusted Publishing |
    |                 | (OIDC auth)        |
    |                 |<-------------------|
    |                 |                    |
    |                 | cargo publish      |
    |                 |------------------->|
    |                 |                    |
    |                 | Create Release     |
    |                 |                    |
```

## Security

- **No token storage**: Trusted Publishing uses OIDC (no secrets in GitHub)
- **Short-lived tokens**: Tokens expire after 30 minutes
- **Audit trail**: All publishes tracked in GitHub Actions logs
- **Manual approval**: Interactive confirmation before push

## Support

For issues or questions:
- [GitHub Issues](https://github.com/tftio/workhelix-cli-common/issues)
- [Repository](https://github.com/tftio/workhelix-cli-common)
