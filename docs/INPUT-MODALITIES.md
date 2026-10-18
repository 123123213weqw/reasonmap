# Input modality evidence

Checked 2026-10-18. Evidence grade: **documented**, not live-accepted.
No paid provider calls were made for these records.

## Current-line image input

These 13 IDs are the current-line image-input set checked on the date above.
Model capability is not a guarantee of account access, regional availability,
endpoint capacity, or consumer-adapter support.

| Exact model | Cataloged protocol | Recorded input | Official evidence |
| --- | --- | --- | --- |
| deepseek-flash | Chat Completions, Responses | text, image | https://api-docs.deepseek.com/guides/vision/ |
| gpt-6.1-sol | Chat Completions, Responses | text, image | https://developers.openai.com/api/docs/models/gpt-6.1-sol |
| gpt-6-astra | Chat Completions, Responses | text, image | https://developers.openai.com/api/docs/models/gpt-6-astra |
| gpt-6-luna | Chat Completions, Responses | text, image | https://developers.openai.com/api/docs/models/gpt-6-luna |
| gemini-3.8-flash | OpenAI-compatible Chat Completions | text, image | https://ai.google.dev/gemini-api/docs/openai |
| qwen3.8-max, qwen3.8-flash | OpenAI-compatible Chat Completions | text, image | [Visual models](https://www.alibabacloud.com/help/en/model-studio/vision-model), [image request format](https://help.aliyun.com/zh/model-studio/vision), [compatible API](https://www.alibabacloud.com/help/en/model-studio/qwen-api-via-openai-chat-completions) |
| kimi-k3 | Chat Completions, Responses | text, image | [Current models](https://platform.kimi.com/docs/models), [Chat images](https://platform.kimi.com/docs/guide/use-kimi-vision-model), [Responses images](https://platform.kimi.com/docs/api/responses) |
| grok-4.7 | Chat Completions | text, image | [Model](https://docs.x.ai/developers/grok-4-7), [images](https://docs.x.ai/developers/model-capabilities/images/understanding), [Chat](https://docs.x.ai/developers/rest-api-reference/inference/chat-completions) |
| claude-fable-5-1, claude-opus-5-5, claude-sonnet-5-5, claude-haiku-5-5 | Native Messages only | text, image | [Current models](https://platform.claude.com/docs/en/models/overview), [vision transport](https://platform.claude.com/docs/en/build-with-claude/vision) |

Claude does not acquire OpenAI-compatible reasoning routes from its vision
metadata. XHarness's current image adapter supports Chat Completions and Responses,
not native Messages: the four Claude entries are catalog facts, not a claim of
direct XHarness support. Grok recommends Responses for new integrations, but this
change does not invent an unrecorded Responses reasoning route; only its existing
documented Chat route is resolved here.

This catalog records the common text/image subset on the listed routes. It does
not enable native audio/video/PDF transports in a consumer, even when a vendor
documents richer model modalities. A model-name suffix is never evidence.

## Compatibility-only records

The existing `gpt-5.4` image record and `deepseek-v4-flash-vision-exp` alias remain
for existing consumers, not as current-line recommendations. DeepSeek documents
that the retired vision-exp alias is served by the latest Flash model; it is not
an additional independent model. The current ID is `deepseek-flash`.

These are deliberately a small verified subset, not a complete vendor list.
A route still requires exact official endpoint, API model ID, and protocol.
A model identity cannot currently express different modality lists per protocol;
only record a list when all cataloged routes for that ID support it. Introduce
route-specific overrides before adding a model whose protocols differ.

Older identities remain unchanged. Reasoning provenance retains its prior
meaning; adding a vision documentation URL does not upgrade live test evidence.
Existing live acceptance artifacts test reasoning with text, not vision.

Modality dates/sources live on the identity, separately from the catalog's
reasoning checked_on/sources. The reasoning snapshot remains dated 2026-10-13.

## Current-line update acceptance

Catalog version `2026-10-18.2` adds 11 image-capability records, giving 13
current-line IDs plus two compatibility-only IDs (15 recorded image identities).
No reasoning profiles, request patches, routes, package version or active model
settings were changed. In particular, this is not a native Messages adapter.

WZU_Server passed 12 Rust catalog tests, one doctest and all-target Clippy with
warnings denied, after rsync of the uncommitted source. The six offline Python
acceptance-runner regressions also passed. No Rust compilation ran on the Mac.
Full remote output is retained outside the repository as
`reasonmap-current-models.log` (SHA-256 `2892c89ed51a41b07a832a4afa3bc8641420f5e10aba91b105a135112834e0af`).
No paid/live image inference or performance acceptance is claimed.
