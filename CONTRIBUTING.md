# Contributing

This is an offline reasoning compatibility catalog, not a universal model SDK.

A new entry must include official HTTPS endpoints, exact API model IDs, protocol,
source URLs and check date, genuine defaults/efforts, and the exact request patch.
Do not infer capability from a model name or equate an accepted JSON field with
verified behavior. Do not claim that a rolling alias pins a version.

Add regression cases for each route and default, unknown models, wrong protocols,
and unsupported effort IDs. Request fragments may only set reasoning extension
fields; never override model, messages, tools, credentials or token budgets.
Include sanitized live evidence where credentials permit, with failures/skips
retained. Provider availability and reasoning quality are separate acceptance gates.

The manifest must remain self-contained, with no dependency on XHarness Host/UI.
The application must retain explicit-configuration priority. Never commit API keys,
private provider responses, application settings, or shell environment dumps.

Follow AGENTS.md for remote Rust compilation. GitHub CI covers Linux, Windows,
and macOS without API credentials. Live tests are opt-in and incur provider usage.
