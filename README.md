# reasonmap

An **offline, documentation-backed reasoning compatibility catalog** for Rust
and language-neutral JSON consumers. Apache-2.0.

It answers: *Which reasoning controls does this exact endpoint + model + protocol
support, and which request fields express them?* It is **not** an LLM SDK, a
proxy, an automatic paid probe, or a tokenizer/model-budget catalog.

## Use

For this unreleased preview (0.4.0 is not a release claim), use the feature branch below.
Production consumers should pin its tested commit rather than a movable branch:

```toml
[dependencies]
reasonmap = { git = "https://github.com/123123213weqw/reasonmap", branch = "feat/input-modalities-20261018" }
```

```rust
use reasonmap::{bundled, Protocol};
let catalog = bundled().expect("valid bundled catalog");
let found = catalog.resolve("https://api.deepseek.com", Protocol::ChatCompletions,
                            "deepseek-flash").expect("documented route");
let patch = found.profile.request_patch(Some("high")).expect("supported effort");
// Apply this fragment through your existing, validated provider adapter.
assert_eq!(patch.expect("patch")["reasoning_effort"], "high");
```

Missing effort uses the catalog's declared default. Unsupported effort returns
an error, never silently maps `medium` to `high`. Unknown route returns `None`.
The library never rewrites model IDs or changes token budgets. Explicit
application configuration must take priority over catalog-derived defaults.

## Initial mappings

| Official model IDs | Reasoning controls | Default |
| --- | --- | --- |
| `deepseek-flash`, `deepseek-v4-pro` | off / low / high / max | high |
| `glm-4.7`, `glm-4.7-flash`, `glm-5`, `glm-5.1` | off / on | on |
| `glm-5.2` | off / high / max | max |
| `glm-5.3`, `glm-5.3-flash` | low / high / max (no off) | max |
| `gpt-6-sol`, `gpt-6-luna`, `gpt-5.6-sol`, `gpt-5.6-terra`, `gpt-5.6-luna` | off / low / medium / high / xhigh / max | medium |
| `gpt-6.1-sol` | low / medium / high / xhigh / max | medium |
| `gpt-6-astra` | low / medium / high / xhigh / max | not documented |
| `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.4-nano` | off / low / medium / high / xhigh | medium / none |
| `gpt-5.1`, `gpt-5` | off / low / medium / high (5 adds minimal) | none / not documented |
| `gpt-5.5-pro` (Responses only) | medium / high / xhigh | high |
| `gemini-3.8-flash`, `gemini-3.1-pro-preview` | low / medium / high | medium / high |
| `gemini-3.6-flash`, `gemini-3.5-flash-lite`, `gemini-3-flash-preview` | minimal / low / medium / high | medium / minimal / high |
| `gemini-2.5-flash`, `gemini-2.5-flash-lite` | off / low / medium / high | dynamic / off |
| `gemini-2.5-pro` | low / medium / high (no off) | dynamic |
| `grok-4.7`, `grok-4.6` | low / medium / high / xhigh (no off) | high |
| `grok-4.5` | low / medium / high (no off) | high |
| `qwen3.8-max`, `qwen3.8-flash` | off / low / medium / xhigh | xhigh |
| `qwen3.7-max`, `qwen3.7-plus`, `qwen3.7-flash` | off / on | on |
| `qwen-plus`, `qwen-flash` | off / on | off |
| `kimi-k2.6` | off / on | on |
| `kimi-k3` | low / high / max (no off) | max |
| `claude-fable-5-1`, `claude-sonnet-5-5` | low / medium / high / xhigh / max (no off) | high |
| `claude-opus-5-5`, `claude-haiku-5-5` | low / medium / high / xhigh / max (no off) | medium |

DeepSeek, OpenAI and Kimi have separate Chat Completions and Responses patches.
GLM, Gemini, xAI and Qwen cover documented Chat Completions endpoints. Anthropic
is matched through its **native Messages protocol** (`anthropic-messages`): its
OpenAI compatibility layer ignores `reasoning_effort`, so effort control is
expressed as `thinking: {type: "adaptive"}` plus `output_config: {effort}`.
The 2026-10-13 OpenAI, Gemini, xAI, Qwen, Kimi and Anthropic additions are
**documentation-backed only**, not live-request tested; DeepSeek and GLM remain
the live-accepted families. Not recorded (insufficient or non-discrete
documentation): OpenAI o-series effort subsets, continuous budget parameters
(`budget_tokens`, `thinking_budget`), Vertex per-project endpoints, and
models whose reasoning cannot be controlled (e.g. `kimi-k2.7-code`).

`deepseek-flash` currently denotes **DeepSeek-V4.1-Flash**. Legacy aliases
`deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` are recorded separately;
they are not separate live model versions. API ID, canonical ID, display name,
and observed version are distinct. A rolling alias does not pin a model version.
This package does not enable image input even when a provider model supports it.

## Data and trust

[`data/catalog.json`](data/catalog.json) is bundled as `CATALOG_JSON`; JavaScript,
TypeScript or Python consumers can read the same versioned data without Rust.
`Catalog::from_json` validates an external snapshot without downloading it.
Facts are matched by exact official endpoint, protocol, and model ID, not vendor
labels or model-name substrings. Unknown proxies do not inherit vendor profiles.

Every route retains source URLs, catalog version and a check date. Documentation
support, fake-wire adapter tests, live request acceptance, and actual control
behavior are different evidence grades; see [acceptance](docs/ACCEPTANCE.md).
No entry alone guarantees provider availability or model reasoning quality.

## Scope and dependencies

Only three direct dependencies: `serde`, `serde_json`, `url` (they have transitive
dependencies). No XHarness Host/UI, Tokio, reqwest, account or credential dependency.
Integration direction is application → catalog. The application owns session
state, settings persistence, discovery, credentials, transport, and UI.

## Tests and contribution

GitHub CI runs locked tests, doctests, formatting and Clippy on Linux, Windows,
and macOS, with standalone package verification on Linux. Live requests are
opt-in using `scripts/live_acceptance.py`; ordinary CI uses no API credentials.
Local development follows the repository's remote Rust compilation policy.

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [LICENSE](LICENSE).
This GitHub source release is not a crates.io publication.

## Input modalities (0.4)

`found.identity.input_modalities` is separate from `ReasoningProfile`. It records
input types documented for an exact matched endpoint, model, and protocol.
An omitted or empty list means **unknown**, not text-only. Accepted values are
`text`, `image`, `audio`, `video`, and `pdf`; a recorded list must include `text`
and contain no duplicates. Old snapshots without this field remain readable.

The first image records cover `deepseek-flash`, its documented legacy
`deepseek-v4-flash-vision-exp` alias, `gpt-5.4`, and `gemini-3.8-flash`.
See [modality evidence](docs/INPUT-MODALITIES.md) for sources and scope.
Other IDs remain unknown, even if their names look vision-capable.
`input_modalities_checked_on` and `input_modalities_sources` provide independent
modality evidence; recording images does not re-date reasoning observations.

This metadata does not implement media transport or turn on an application's
image input. Consumers must intersect it with adapter support and explicit
user configuration. It does not advertise image generation. Audio/video/PDF
schema values reserve catalog vocabulary, not XHarness implementations.
