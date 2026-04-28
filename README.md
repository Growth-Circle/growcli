# Grow CLI

[![npm version](https://img.shields.io/npm/v/@growthcircle/growcli?color=cb3837&label=npm)](https://www.npmjs.com/package/@growthcircle/growcli)
[![build](https://img.shields.io/github/actions/workflow/status/Growth-Circle/growcli/npm-publish.yml?label=build)](https://github.com/Growth-Circle/growcli/actions/workflows/npm-publish.yml)
[![downloads](https://img.shields.io/npm/dm/@growthcircle/growcli?color=blue&label=installs)](https://www.npmjs.com/package/@growthcircle/growcli)
[![community](https://img.shields.io/endpoint?url=https://growthcircle.id/api/public/badge/members&label=community)](https://growthcircle.id)
[![license](https://img.shields.io/github/license/Growth-Circle/growcli)](./LICENSE)

GrowthCircle coding agent for your terminal. Built on [OpenAI Codex CLI](https://github.com/openai/codex), powered by models from [GrowthCircle AI](https://growthcircle.id/app/ai) — one API key, free and paid models included.

## Install

```shell
npm install -g @growthcircle/growcli
```

Supported platforms: **Linux x64** · **macOS x64** · **macOS arm64** · **Windows x64**

## Quick start

```shell
growcli                              # interactive agent (prompts for API key on first run)
growcli -m MODEL_ID                  # use a specific model
growcli exec "explain this repo"     # one-shot task
```

Set your API key (optional — `growcli` prompts on first run if unset):

```shell
export GC_API_KEY="your_growthcircle_api_key"
```

## Install from source

```shell
git clone https://github.com/Growth-Circle/growcli.git
cd growcli/codex-rs
CODEX_SKIP_VENDORED_BWRAP=1 cargo install --path cli --bin growcli --locked
```

Or run without installing:

```shell
cd growcli/codex-rs
cargo run --bin growcli -- --help
```

## GrowthCircle provider

Grow CLI defaults to the built-in `growthcircle` provider:

| Setting | Value |
|---------|-------|
| Base URL | `https://ai.growthcircle.id/v1` |
| Auth | `Authorization: Bearer $GC_API_KEY` |
| Agent endpoint | `POST /v1/responses` |
| Chat endpoint | `POST /v1/chat/completions` |
| Image endpoint | `POST /v1/images/generations` |

All models available to your account (free and paid) are selected with `-m MODEL_ID` or in config:

```toml
model_provider = "growthcircle"
model = "MODEL_ID"
```

Save in `~/.codex/config.toml` or pass on the command line.

## Documentation

- [GrowthCircle setup](./docs/growthcircle.md)
- [Installing & system requirements](./docs/install.md)
- [Contributing](./docs/contributing.md)

## Upstream

Grow CLI tracks upstream [OpenAI Codex CLI](https://github.com/openai/codex) where practical. Fork-specific changes: GrowthCircle provider defaults, `GC_API_KEY` auth, and the `growcli` / `grow` commands.

## License

Apache-2.0 — see [LICENSE](./LICENSE). Upstream attribution preserved in [NOTICE](./NOTICE).
