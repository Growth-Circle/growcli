# Sample configuration

Grow CLI defaults to GrowthCircle AI, but you can keep multiple providers in
one `~/.codex/config.toml` and switch between them with profiles.

Use this sample as a starting point:

- [OpenAI-compatible chat providers](./examples/openai-compatible-chat.toml)

That file includes:

- GrowthCircle as the default provider.
- MiniMax and Z.ai as OpenAI-compatible Chat Completions providers.
- A generic custom provider template for any service that accepts an OpenAI SDK
  style `base_url`, API key, and model ID.

Keep API keys in environment variables, for example:

```shell
export GC_API_KEY="your_growthcircle_api_key"
export MINIMAX_API_KEY="sk-..."
export ZAI_API_KEY="sk-..."
export CUSTOM_OPENAI_API_KEY="sk-..."
```

Run with a profile:

```shell
growcli --profile growthcircle
growcli --profile minimax
growcli --profile zai
growcli --profile custom_chat
```

For the upstream Codex sample configuration reference, see
https://developers.openai.com/codex/config-sample.
