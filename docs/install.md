## Installing & building

### System requirements

| Requirement                 | Details                                            |
| --------------------------- | -------------------------------------------------- |
| Operating systems           | macOS 12+, Ubuntu 20.04+/Debian 10+, or Windows 11 |
| Git (optional, recommended) | 2.23+ for built-in PR helpers                      |
| RAM                         | 4-GB minimum (8-GB recommended)                    |

### DotSlash

Future GitHub releases can contain a [DotSlash](https://dotslash-cli.com/) file for Grow CLI named `growcli`. Using a DotSlash file makes it possible to make a lightweight commit to source control to ensure all contributors use the same version of an executable, regardless of what platform they use for development.

### Build from source

```bash
# Clone the repository and navigate to the root of the Cargo workspace.
git clone https://github.com/Growth-Circle/growcli.git
cd growcli/codex-rs

# Install the Rust toolchain, if necessary.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup component add rustfmt
rustup component add clippy
# Install helper tools used by the workspace justfile:
cargo install just
# Optional: install nextest for the `just test` helper
cargo install --locked cargo-nextest

# Build Grow CLI.
cargo build

# Launch the TUI with a sample prompt.
cargo run --bin growcli -- "explain this codebase to me"

# After making changes, use the root justfile helpers (they default to codex-rs):
just fmt
just fix -p <crate-you-touched>

# Run the relevant tests (project-specific is fastest), for example:
cargo test -p codex-tui
# If you have cargo-nextest installed, `just test` runs the test suite via nextest:
just test
# Avoid `--all-features` for routine local runs because it increases build
# time and `target/` disk usage by compiling additional feature combinations.
# If you specifically want full feature coverage, use:
cargo test --all-features
```

### Install from npm

Install Grow CLI from npm:

```bash
npm install -g @growthcircle/growcli
growcli --version
growcli
```

The npm workflow publishes native payloads for Linux x64, macOS x64, macOS
arm64 / Apple Silicon, and Windows x64. The root package uses npm optional
dependencies to install the right native payload for the user's machine.

Maintainers publish the package with the manual GitHub Actions `npm-publish`
workflow after adding the `NPM_TOKEN` repository secret.

### Install from source

Install directly from this repository when developing or testing unpublished
changes:

```bash
git clone https://github.com/Growth-Circle/growcli.git
cd growcli/codex-rs
CODEX_SKIP_VENDORED_BWRAP=1 cargo install --path cli --bin growcli --locked
growcli --version
growcli
```

## Tracing / verbose logging

Codex is written in Rust, so it honors the `RUST_LOG` environment variable to configure its logging behavior.

The TUI defaults to `RUST_LOG=codex_core=info,codex_tui=info,codex_rmcp_client=info` and log messages are written to `~/.codex/log/codex-tui.log` by default. For a single run, you can override the log directory with `-c log_dir=...` (for example, `-c log_dir=./.codex-log`).

```bash
tail -F ~/.codex/log/codex-tui.log
```

By comparison, the non-interactive mode (`growcli exec`) defaults to `RUST_LOG=error`, but messages are printed inline, so there is no need to monitor a separate file.

See the Rust documentation on [`RUST_LOG`](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) for more information on the configuration options.
