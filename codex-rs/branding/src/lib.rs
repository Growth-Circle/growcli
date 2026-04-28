//! Fork-specific branding constants for Grow CLI.
//!
//! Centralises every user-visible string that differs from upstream Codex so
//! that upstream merges only conflict in this one file.

/// Display name shown in the TUI, status bar, and onboarding.
pub const APP_NAME: &str = "Grow CLI";

/// Primary binary name used in help text, tooltips, and error messages.
pub const BIN_NAME: &str = "growcli";

/// npm package scope + name for update prompts and install commands.
pub const NPM_PACKAGE: &str = "@growthcircle/growcli";

/// GitHub releases URL for the update prompt.
pub const RELEASE_URL: &str = "https://github.com/Growth-Circle/growcli/releases/latest";

/// GitHub repository URL shown in the status card "forked from" line.
pub const REPO_URL: &str = "https://github.com/Growth-Circle/growcli";
