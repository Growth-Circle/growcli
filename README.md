# Grow CLI

Grow CLI is a GrowthCircle-focused fork of [OpenAI Codex CLI](https://github.com/openai/codex). It keeps the local coding-agent workflow from Codex and adds a built-in `growthcircle` model provider so GrowthCircle users can use models from https://growthcircle.id/app/ai with one API key.

This repository is open source under Apache-2.0. Upstream attribution and notices are preserved in `LICENSE` and `NOTICE`.

## Quickstart

Install Grow CLI from npm:

```shell
npm install -g @growthcircle/growcli
growcli --version
growcli
```

Package:
[`@growthcircle/growcli`](https://www.npmjs.com/package/@growthcircle/growcli).
The same install command is used on supported Linux, macOS, and Windows
machines.

Current npm targets:

- Linux x64
- macOS x64
- macOS arm64 / Apple Silicon
- Windows x64

Install from source:

```shell
git clone https://github.com/Growth-Circle/growcli.git
cd growcli/codex-rs
CODEX_SKIP_VENDORED_BWRAP=1 cargo install --path cli --bin growcli --locked
growcli --version
growcli
```

Maintainers publish the npm package from GitHub Actions with the manual
`npm-publish` workflow. It builds native npm payloads for Linux, macOS, and
Windows, publishes them first, then publishes the root package that selects the
right native payload through npm optional dependencies. The repository needs an
`NPM_TOKEN` secret with publish access to the `@growthcircle` npm scope.

If `GC_API_KEY` is not set yet, Grow CLI asks for the GrowthCircle API key on
startup, validates it, saves it locally, and loads the free or paid models
available to that key.

Run from source without installing:

```shell
git clone https://github.com/Growth-Circle/growcli.git
cd growcli/codex-rs
cargo run --bin growcli -- --help
```

Set your GrowthCircle API key:

```shell
export GC_API_KEY="your_growthcircle_api_key"
```

This step is optional for interactive users because `growcli` can prompt for
the key on first run.

Start the interactive coding agent:

```shell
cargo run --bin growcli --
```

Use a specific model from the GrowthCircle AI dashboard:

```shell
cargo run --bin growcli -- -m MODEL_ID
```

Run a one-shot task:

```shell
cargo run --bin growcli -- exec -m MODEL_ID "explain this repository"
```

## GrowthCircle Provider

Grow CLI defaults to the built-in `growthcircle` provider:

```toml
model_provider = "growthcircle"
```

Provider details:

- Base URL: `https://ai.growthcircle.id/v1`
- Auth header: `Authorization: Bearer $GC_API_KEY` or the API key saved during
  first-run setup
- Main endpoint used by the agent: `POST /v1/responses`
- OpenAI-compatible chat endpoint for integrations: `POST /v1/chat/completions`
- Image endpoint for image tools: `POST /v1/images/generations`

All GrowthCircle models, including free and paid models available to your account, are selected by model ID with `-m MODEL_ID` or `model = "MODEL_ID"` in config.

## Configuration

Grow CLI uses the same config system as Codex. To pin a model:

```toml
model_provider = "growthcircle"
model = "MODEL_ID"
```

Save this in `~/.codex/config.toml` for your user account, or pass overrides on the command line.

## Upstream

Grow CLI tracks upstream OpenAI Codex CLI where practical. The main fork-specific changes are the GrowthCircle provider defaults, `GC_API_KEY` authentication, and the `growcli` command and `grow` shortcut.

Helpful docs:

- [GrowthCircle setup](./docs/growthcircle.md)
- [Installing, building, and system requirements](./docs/install.md)
- [Contributing](./docs/contributing.md)
