#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallContext;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::StandalonePlatform;

/// Update action the CLI should perform after the TUI exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Update via `npm install -g @growthcircle/growcli@latest`.
    NpmGlobalLatest,
    /// Update via `bun install -g @growthcircle/growcli@latest`.
    BunGlobalLatest,
    /// Update via npm. Grow CLI is published through npm, not Homebrew.
    BrewUpgrade,
    /// Update via npm. Grow CLI is published through npm, not the Codex standalone installer.
    StandaloneUnix,
    /// Update via npm. Grow CLI is published through npm, not the Codex standalone installer.
    StandaloneWindows,
}

impl UpdateAction {
    #[cfg(any(not(debug_assertions), test))]
    pub(crate) fn from_install_context(context: &InstallContext) -> Option<Self> {
        match context {
            InstallContext::Npm => Some(UpdateAction::NpmGlobalLatest),
            InstallContext::Bun => Some(UpdateAction::BunGlobalLatest),
            InstallContext::Brew => Some(UpdateAction::BrewUpgrade),
            InstallContext::Standalone { platform, .. } => Some(match platform {
                StandalonePlatform::Unix => UpdateAction::StandaloneUnix,
                StandalonePlatform::Windows => UpdateAction::StandaloneWindows,
            }),
            InstallContext::Other => None,
        }
    }

    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(self) -> (&'static str, &'static [&'static str]) {
        match self {
            UpdateAction::NpmGlobalLatest
            | UpdateAction::BrewUpgrade
            | UpdateAction::StandaloneUnix
            | UpdateAction::StandaloneWindows => {
                ("npm", &["install", "-g", "@growthcircle/growcli@latest"])
            }
            UpdateAction::BunGlobalLatest => {
                ("bun", &["install", "-g", "@growthcircle/growcli@latest"])
            }
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command).chain(args.iter().copied()))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn get_update_action() -> Option<UpdateAction> {
    UpdateAction::from_install_context(InstallContext::current())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn maps_install_context_to_update_action() {
        let native_release_dir = PathBuf::from("/tmp/native-release");

        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Other),
            None
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Npm),
            Some(UpdateAction::NpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Bun),
            Some(UpdateAction::BunGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Brew),
            Some(UpdateAction::BrewUpgrade)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Standalone {
                platform: StandalonePlatform::Unix,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            }),
            Some(UpdateAction::StandaloneUnix)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Standalone {
                platform: StandalonePlatform::Windows,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            }),
            Some(UpdateAction::StandaloneWindows)
        );
    }

    #[test]
    fn standalone_update_commands_rerun_latest_installer() {
        assert_eq!(
            UpdateAction::StandaloneUnix.command_args(),
            (
                "npm",
                &["install", "-g", "@growthcircle/growcli@latest"][..],
            )
        );
        assert_eq!(
            UpdateAction::StandaloneWindows.command_args(),
            (
                "npm",
                &["install", "-g", "@growthcircle/growcli@latest"][..],
            )
        );
    }
}
