use reasonmap::{bundled, Catalog, Protocol, CATALOG_JSON};
use serde_json::{json, Value};
fn altered(edit: impl FnOnce(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(CATALOG_JSON).unwrap();
    edit(&mut v);
    v.to_string()
}
#[test]
fn bundled_routes_profiles_and_identities_validate() {
    let catalog = bundled().unwrap();
    let data: Value = serde_json::from_str(CATALOG_JSON).unwrap();
    assert_eq!(catalog.version(), data["version"]);
    for route in data["routes"].as_array().unwrap() {
        let protocol = serde_json::from_value(route["api"].clone()).unwrap();
        for base in route["base_urls"].as_array().unwrap() {
            for id in route["models"].as_array().unwrap() {
                let found = catalog
                    .resolve(base.as_str().unwrap(), protocol, id.as_str().unwrap())
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(found.profile).unwrap(),
                    data["profiles"][route["profile"].as_str().unwrap()]
                );
                assert!(!found.sources.is_empty());
            }
        }
    }
}
#[test]
fn flash_41_identity_and_legacy_aliases_do_not_invent_new_api_ids() {
    for id in [
        "deepseek-flash",
        "deepseek-v4-flash",
        "deepseek-v4-flash-vision-exp",
    ] {
        let found = bundled()
            .unwrap()
            .resolve("https://api.deepseek.com", Protocol::ChatCompletions, id)
            .unwrap();
        assert_eq!(found.identity.canonical_id, "deepseek-flash");
        assert_eq!(found.identity.snapshot_version, "DeepSeek-V4.1-Flash");
        assert_eq!(found.identity.legacy_alias, id != "deepseek-flash");
        assert_eq!(found.profile.default_effort.as_deref(), Some("high"));
    }
    assert!(bundled()
        .unwrap()
        .resolve(
            "https://api.deepseek.com",
            Protocol::ChatCompletions,
            "deepseek-v4.1-flash"
        )
        .is_none());
}
#[test]
fn unknown_endpoints_models_and_wrong_protocol_never_inherit_profiles() {
    for base in [
        "http://api.deepseek.com",
        "https://api.deepseek.com.evil.test",
        "https://proxy.test/v1",
        "https://api.deepseek.com/proxy",
        "https://api.deepseek.com:8443",
        "https://user@api.deepseek.com",
        "https://api.deepseek.com?key=secret",
        "https://api.deepseek.com/#fragment",
    ] {
        assert!(
            bundled()
                .unwrap()
                .resolve(base, Protocol::ChatCompletions, "deepseek-flash")
                .is_none(),
            "{base}"
        );
    }
    assert!(bundled()
        .unwrap()
        .resolve(
            "https://api.deepseek.com/v1/",
            Protocol::ChatCompletions,
            "deepseek-flash"
        )
        .is_some());
    assert!(bundled()
        .unwrap()
        .resolve(
            "https://api.deepseek.com",
            Protocol::ChatCompletions,
            "DeepSeek Flash"
        )
        .is_none());
    assert!(bundled()
        .unwrap()
        .resolve("https://api.z.ai/api/v1", Protocol::Responses, "glm-5.3")
        .is_none());
}
#[test]
fn chat_and_responses_fields_are_distinct_and_defaults_are_real() {
    let chat = bundled()
        .unwrap()
        .resolve(
            "https://api.deepseek.com",
            Protocol::ChatCompletions,
            "deepseek-flash",
        )
        .unwrap();
    let responses = bundled()
        .unwrap()
        .resolve(
            "https://api.deepseek.com",
            Protocol::Responses,
            "deepseek-flash",
        )
        .unwrap();
    assert_eq!(
        chat.profile.request_patch(Some("off")).unwrap().unwrap()["thinking"],
        json!({"type":"disabled"})
    );
    assert_eq!(
        responses
            .profile
            .request_patch(Some("off"))
            .unwrap()
            .unwrap()["reasoning"],
        json!({"effort":"none"})
    );
    assert_eq!(
        chat.profile.request_patch(None).unwrap(),
        chat.profile.request_patch(Some("high")).unwrap()
    );
    assert!(
        chat.profile.request_patch(Some("medium")).is_err(),
        "do not invent a normalized level"
    );
}
#[test]
fn glm_model_specific_levels_and_forced_reasoning_are_not_universal_low_medium_high() {
    for (model, ids, default) in [
        ("glm-4.7", vec!["off", "on"], "on"),
        ("glm-5.2", vec!["off", "high", "max"], "max"),
        ("glm-5.3", vec!["low", "high", "max"], "max"),
    ] {
        let found = bundled()
            .unwrap()
            .resolve(
                "https://open.bigmodel.cn/api/paas/v4",
                Protocol::ChatCompletions,
                model,
            )
            .unwrap();
        assert_eq!(
            found
                .profile
                .efforts
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(found.profile.default_effort.as_deref(), Some(default));
        if model == "glm-5.3" {
            assert!(found.profile.request_patch(Some("off")).is_err());
        }
    }
}
#[test]
fn documented_families_use_their_documented_control_fields() {
    let catalog = bundled().unwrap();
    let astra = catalog
        .resolve(
            "https://api.openai.com/v1",
            Protocol::ChatCompletions,
            "gpt-6-astra",
        )
        .unwrap();
    assert!(astra.profile.request_patch(Some("off")).is_err());
    assert_eq!(
        astra.profile.request_patch(Some("high")).unwrap().unwrap()["reasoning_effort"],
        "high"
    );
    assert!(astra.profile.default_effort.is_none());
    let sol = catalog
        .resolve(
            "https://api.openai.com/v1",
            Protocol::Responses,
            "gpt-6.1-sol",
        )
        .unwrap();
    assert!(sol.profile.request_patch(Some("off")).is_err());
    assert_eq!(
        sol.profile.request_patch(Some("low")).unwrap().unwrap()["reasoning"],
        json!({"effort": "low"})
    );
    let g55 = catalog
        .resolve("https://api.openai.com/v1", Protocol::Responses, "gpt-5.5")
        .unwrap();
    assert_eq!(
        g55.profile.request_patch(Some("off")).unwrap().unwrap()["reasoning"],
        json!({"effort": "none"})
    );
    let sonnet = catalog
        .resolve(
            "https://api.anthropic.com",
            Protocol::AnthropicMessages,
            "claude-sonnet-5-5",
        )
        .unwrap();
    let sonnet_patch = sonnet
        .profile
        .request_patch(Some("xhigh"))
        .unwrap()
        .unwrap();
    assert_eq!(sonnet_patch["thinking"], json!({"type": "adaptive"}));
    assert_eq!(sonnet_patch["output_config"], json!({"effort": "xhigh"}));
    let gemini = catalog
        .resolve(
            "https://generativelanguage.googleapis.com/v1beta/openai",
            Protocol::ChatCompletions,
            "gemini-3.8-flash",
        )
        .unwrap();
    assert!(gemini.profile.request_patch(Some("minimal")).is_err());
    assert_eq!(
        gemini.profile.request_patch(Some("low")).unwrap().unwrap()["reasoning_effort"],
        "low"
    );
    let grok = catalog
        .resolve("https://api.x.ai/v1", Protocol::ChatCompletions, "grok-4.7")
        .unwrap();
    assert!(grok.profile.request_patch(Some("off")).is_err());
    let qwen = catalog
        .resolve(
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            Protocol::ChatCompletions,
            "qwen-plus",
        )
        .unwrap();
    assert_eq!(
        qwen.profile.request_patch(None).unwrap().unwrap()["enable_thinking"],
        false
    );
    let k3 = catalog
        .resolve("https://api.moonshot.ai/v1", Protocol::Responses, "kimi-k3")
        .unwrap();
    assert_eq!(
        k3.profile.request_patch(Some("low")).unwrap().unwrap()["reasoning"],
        json!({"effort": "low"})
    );
    assert!(k3.profile.request_patch(Some("off")).is_err());
    assert!(
        catalog
            .resolve(
                "https://api.anthropic.com",
                Protocol::ChatCompletions,
                "claude-sonnet-5-5"
            )
            .is_none(),
        "anthropic control is native-protocol only"
    );
}
#[test]
fn malformed_duplicate_or_unproven_catalog_entries_are_rejected() {
    for input in [
        altered(|v| v["profiles"]["glm-5.3"]["default_effort"] = json!("off")),
        altered(|v| {
            v["profiles"]["glm-5.3"]["efforts"][0]["request_patch"] = json!({"messages":[]})
        }),
        altered(|v| v["profiles"]["glm-5.3"]["efforts"][0]["id"] = json!("max")),
        altered(|v| v["profiles"]["glm-5.3"]["efforts"] = json!([])),
        altered(|v| v["routes"][0]["sources"] = json!([])),
        altered(|v| v["routes"][0]["profile"] = json!("missing")),
        altered(|v| v["routes"][0]["models"] = json!(["not-declared"])),
        altered(|v| v["routes"][0]["base_urls"] = json!(["http://api.deepseek.com"])),
        altered(|v| v["routes"][0]["api"] = json!("google-gemini")),
        altered(|v| {
            let route = v["routes"][0].clone();
            v["routes"].as_array_mut().unwrap().push(route)
        }),
        altered(|v| v["identities"]["deepseek-flash"]["snapshot_version"] = json!("")),
        altered(|v| {
            v["identities"]["deepseek-flash"]["input_modalities"] = json!(["text", "image", "text"])
        }),
        altered(|v| v["identities"]["deepseek-flash"]["input_modalities"] = json!(["image"])),
        altered(|v| {
            v["identities"]["deepseek-flash"]["input_modalities"] = json!(["text", "hologram"])
        }),
        altered(|v| v["unexpected"] = json!(true)),
    ] {
        assert!(Catalog::from_json(&input).is_err());
    }
    assert!(Catalog::from_json(&"x".repeat(2 * 1024 * 1024 + 1)).is_err());
}
#[test]
fn snapshots_without_modalities_stay_valid_and_empty_means_unknown() {
    let legacy = altered(|v| {
        v["identities"]["deepseek-flash"]
            .as_object_mut()
            .unwrap()
            .remove("input_modalities");
    });
    let catalog = Catalog::from_json(&legacy).unwrap();
    let found = catalog
        .resolve(
            "https://api.deepseek.com",
            Protocol::ChatCompletions,
            "deepseek-flash",
        )
        .unwrap();
    assert!(found.identity.input_modalities.is_empty());
}

#[test]
fn documented_image_inputs_are_separate_from_reasoning_and_exactly_routed() {
    let catalog = bundled().unwrap();
    for (base, protocol, model) in [
        (
            "https://api.deepseek.com",
            Protocol::ChatCompletions,
            "deepseek-flash",
        ),
        (
            "https://api.deepseek.com/v1/",
            Protocol::Responses,
            "deepseek-v4-flash-vision-exp",
        ),
        (
            "https://api.openai.com/v1",
            Protocol::ChatCompletions,
            "gpt-5.4",
        ),
        ("https://api.openai.com/v1", Protocol::Responses, "gpt-5.4"),
        (
            "https://generativelanguage.googleapis.com/v1beta/openai",
            Protocol::ChatCompletions,
            "gemini-3.8-flash",
        ),
    ] {
        let found = catalog.resolve(base, protocol, model).unwrap();
        assert_eq!(found.identity.input_modalities, ["text", "image"]);
        assert!(!found.profile.efforts.is_empty());
        assert_eq!(
            found.identity.input_modalities_checked_on.as_deref(),
            Some("2026-10-18")
        );
        assert_eq!(found.checked_on, "2026-10-13");
        assert!(found
            .identity
            .input_modalities_sources
            .iter()
            .any(|s| s.contains("vision") || s.contains("gpt-5.4") || s.contains("/openai")));
    }
    assert!(catalog
        .resolve("https://proxy.test/v1", Protocol::Responses, "gpt-5.4")
        .is_none());
    assert!(catalog
        .resolve(
            "https://generativelanguage.googleapis.com/v1beta/openai",
            Protocol::Responses,
            "gemini-3.8-flash"
        )
        .is_none());
    assert!(catalog
        .resolve(
            "https://api.deepseek.com",
            Protocol::ChatCompletions,
            "deepseek-v4-pro"
        )
        .unwrap()
        .identity
        .input_modalities
        .is_empty());
}

#[test]
fn modality_evidence_is_required_without_relabelling_reasoning_evidence() {
    for field in ["input_modalities_checked_on", "input_modalities_sources"] {
        let value = altered(|v| {
            v["identities"]["deepseek-flash"]
                .as_object_mut()
                .unwrap()
                .remove(field);
        });
        assert!(Catalog::from_json(&value).is_err());
    }
    let value = altered(|v| {
        v["identities"]["deepseek-flash"]["input_modalities_sources"] =
            json!(["http://source.test"]);
    });
    assert!(Catalog::from_json(&value).is_err());
}
