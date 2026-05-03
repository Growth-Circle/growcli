# GrowthCircle Provider

Grow CLI includes a built-in `growthcircle` provider for GrowthCircle AI.

## API Key

Create or copy your API key from:

```text
https://growthcircle.id/app/ai
```

Then export it before running Grow CLI:

```shell
export GC_API_KEY="your_growthcircle_api_key"
```

If `GC_API_KEY` is not set, run `growcli` and paste the key when prompted. Grow
CLI validates the key against GrowthCircle, saves it locally, and loads the
models available to that key, including free and paid models.

Grow CLI sends it as:

```http
Authorization: Bearer GC_API_KEY
```

When the key is entered in first-run setup, Grow CLI stores it locally and uses
the same bearer auth header.

## Default Provider

The fork defaults to:

```toml
model_provider = "growthcircle"
```

The built-in provider uses:

```toml
[model_providers.growthcircle]
name = "GrowthCircle"
base_url = "https://ai.growthcircle.id/v1"
env_key = "GC_API_KEY"
wire_api = "responses"
requires_openai_auth = true
```

`growthcircle` is a reserved built-in provider ID. If you need a staging or custom GrowthCircle endpoint, create a new provider ID:

```toml
model_provider = "growthcircle-staging"

[model_providers.growthcircle-staging]
name = "GrowthCircle Staging"
base_url = "https://ai.growthcircle.id/v1"
env_key = "GC_API_KEY"
wire_api = "responses"
requires_openai_auth = true
```

## Model Selection

Use any model ID available to your GrowthCircle account:

```shell
growcli -m MODEL_ID
growcli exec -m MODEL_ID "summarize this codebase"
```

Or pin it in `~/.codex/config.toml`:

```toml
model_provider = "growthcircle"
model = "MODEL_ID"
```

## Supported OpenAI-Compatible Endpoints

GrowthCircle exposes these endpoints with the same `GC_API_KEY`:

- `POST /v1/responses` for modern Responses API workflows and reasoning.
- `POST /v1/chat/completions` for chat, coding assistants, agents, and tool integrations that use OpenAI chat format.
- `POST /v1/images/generations` for image generation.

Grow CLI currently uses the Responses API path for the coding agent.

## Third-Party OpenAI-Compatible APIs

GrowthCircle remains the default coding-agent backend. For providers that only
support OpenAI Chat Completions, create a separate provider with
`wire_api = "chat"` and select it explicitly:

```toml
model_provider = "custom-openai-chat"
model = "MODEL_ID_FROM_PROVIDER"

[model_providers.custom-openai-chat]
name = "Custom OpenAI-compatible Chat"
base_url = "https://provider.example/v1"
env_key = "CUSTOM_OPENAI_API_KEY"
wire_api = "chat"
requires_openai_auth = false
```

Then run:

```shell
export CUSTOM_OPENAI_API_KEY="sk-..."
growcli -m MODEL_ID_FROM_PROVIDER
```

You can add multiple custom providers and switch between them with profiles or
by changing `model_provider`. GrowthCircle remains available as
`model_provider = "growthcircle"` with `GC_API_KEY`.
