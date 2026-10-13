# reasonmap

An **offline, documentation-backed reasoning compatibility catalog** for Rust
and language-neutral JSON consumers. Apache-2.0.

It answers: *Which reasoning controls does this exact endpoint + model + protocol
support, and which request fields express them?* It is **not** an LLM SDK, a
proxy, an automatic paid probe, or a tokenizer/model-budget catalog.

## Use

Until a crates.io release is published:

```toml
[dependencies]
reasonmap = { git = "https://github.com/123123213weqw/reasonmap", tag = "v0.2.0" }
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

DeepSeek has separate Chat Completions and Responses patches. GLM initially
covers documented Chat Completions endpoints at BigModel and Z.ai.

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
