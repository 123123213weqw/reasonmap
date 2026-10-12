# Acceptance — 2026-10-12

This release is an offline reasoning catalog, not a certification of model quality.
Evidence is intentionally separated into four grades:

1. **Documented**: exact endpoint/model/protocol/default/patch backed by sources
   in the catalog. This includes all bundled mappings.
2. **Contract-tested**: bundled and malformed snapshots, exact endpoint guards,
   defaults, opaque effort IDs, aliases and protocol-specific fields. Six Rust
   contract tests and one doctest pass in the extracted standalone checkout.
3. **Adapter-tested**: XHarness's existing Rust adapter sends the expected wire
   fragment. Integration acceptance passed 88 Host unit tests (one unrelated
   opt-in external-download test skipped), 17 settings/RPC tests, two fake-HTTP
   wire tests, Clippy and Chromium/WebKit regressions. These are integration
   evidence; XHarness is NOT a package dependency.
4. **Live request acceptance / observed behavior**: the synthetic prompt computes
   17×19; clean completion and exact answer are required, and reported usage and
   observable reasoning are recorded separately. A 200 alone is insufficient.

## Live scope and results

- DeepSeek official `https://api.deepseek.com`: **24/24 request cases pass**,
  covering Flash (observed V4.1), Pro, Chat Completions and Responses, each effort
  and catalog default; legacy aliases are checked at the default effort only.
  Two alias/Responses transport failures passed targeted retry. First attempts
  remain embedded in [the evidence](live-deepseek-2026-10-12.json).
- GLM official BigModel `https://open.bigmodel.cn/api/paas/v4`: **23/24 request
  cases pass** across all seven declared IDs and their efforts/defaults.
  `glm-4.7-flash` catalog-default remains blocked by HTTP 429 / numeric code 1305
  after two retries. Its explicit off/on requests pass, but that does not mark
  the blocked default request as passed. See [the evidence](live-glm-2026-10-12.json).
- Z.ai and the BigModel/Z.ai Coding Plan endpoints are **documentation-backed,
  not live-tested** in this run. A credential is never forwarded to a different
  endpoint after an authentication failure.
- GLM Responses, other model IDs, proxies, streaming/reconnect, visual input,
  tool-call reasoning continuation, and reasoning-quality ranking are **not
  accepted by this run**. They are not claimed as tested features here.

Off responses can omit the reasoning field; this is recorded as not-observable,
not guessed as proof of internal behavior. One GLM-5.3/high response also lacked
observable reasoning, despite clean request acceptance. Even a visible reasoning
response does not prove the provider applied a different amount of computation
for low/high/max. No quality ranking is asserted.

## Reproduce

Run ordinary locked Rust tests through the remote development host per AGENTS.md,
or use the credential-free three-platform GitHub CI. The standalone manifest has
only serde, serde_json and url as direct dependencies. Remote package verification
passes; no local Rust compilation was used.

For an opt-in live check (Python 3 standard library only):

```sh
python3 scripts/live_acceptance.py --family deepseek \
  --base-url https://api.deepseek.com --key-stdin --workers 1 \
  --output /tmp/catalog-live.json
```

Supply a key via protected stdin (disable terminal echo) or `CATALOG_API_KEY`,
never a command-line argument. For BigModel use `--family glm` and its official
base URL. Output contains counters/statuses, no API key or response bodies. Automatic
HTTP redirects are disabled: credentials never follow a redirected request.
`--retry-failed /tmp/catalog-live.json` checks only failed cases and preserves
prior attempts. Do not put credentials in shell history or commit them.

Each request caps max output at 2,048 tokens; no tools or private chat data are
sent. Usage counters are recorded, not a measured monetary cost. Ordinary CI
never performs these paid requests. Do not interpret the arithmetic fixture as
a realistic complex-task, latency, or quality benchmark.

## Release scope

GitHub source distribution is permitted with these explicitly retained limits.
This is not a claim of all-provider live certification or a crates.io release.
Three-platform CI results and release assets are authoritative on the exact tag;
local/remote preflight success does not substitute for GitHub CI completion.
