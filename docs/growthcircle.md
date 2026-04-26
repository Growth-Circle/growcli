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

Grow CLI sends it as:

```http
Authorization: Bearer GC_API_KEY
```

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
requires_openai_auth = false
```

`growthcircle` is a reserved built-in provider ID. If you need a staging or custom GrowthCircle endpoint, create a new provider ID:

```toml
model_provider = "growthcircle-staging"

[model_providers.growthcircle-staging]
name = "GrowthCircle Staging"
base_url = "https://ai.growthcircle.id/v1"
env_key = "GC_API_KEY"
wire_api = "responses"
requires_openai_auth = false
```

## Model Selection

Use any model ID available to your GrowthCircle account:

```shell
grow -m MODEL_ID
grow exec -m MODEL_ID "summarize this codebase"
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

