# Input modality evidence

Checked 2026-10-18. Evidence grade: **documented**, not live-accepted.
No paid provider calls were made for these records.

| Exact model | Protocol | Recorded input | Official evidence |
| --- | --- | --- | --- |
| deepseek-flash | Chat Completions, Responses | text, image | https://api-docs.deepseek.com/guides/vision/ |
| deepseek-v4-flash-vision-exp | Chat Completions, Responses | text, image | Same guide explicitly documents the legacy alias |
| gpt-5.4 | Chat Completions, Responses | text, image | https://developers.openai.com/api/docs/models/gpt-5.4 |
| gemini-3.8-flash | OpenAI-compatible Chat Completions | text, image | https://ai.google.dev/gemini-api/docs/openai |

These are deliberately a small verified subset, not a complete vendor list.
A route still requires exact official endpoint, API model ID, and protocol.
A model identity cannot currently express different modality lists per protocol;
only record a list when all cataloged routes for that ID support it. Introduce
route-specific overrides before adding a model whose protocols differ.

Older identities remain unchanged. Reasoning provenance retains its prior
meaning; adding a vision documentation URL does not upgrade live test evidence.
Existing live acceptance artifacts test reasoning with text, not vision.
