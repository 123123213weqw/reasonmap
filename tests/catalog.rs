use serde_json::{json, Value};
use xharness_model_catalog::{bundled, Catalog, Protocol, CATALOG_JSON};
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
        altered(|v| v["routes"][0]["api"] = json!("anthropic-messages")),
        altered(|v| {
            let route = v["routes"][0].clone();
            v["routes"].as_array_mut().unwrap().push(route)
        }),
        altered(|v| v["identities"]["deepseek-flash"]["snapshot_version"] = json!("")),
        altered(|v| v["unexpected"] = json!(true)),
    ] {
        assert!(Catalog::from_json(&input).is_err());
    }
    assert!(Catalog::from_json(&"x".repeat(2 * 1024 * 1024 + 1)).is_err());
}
