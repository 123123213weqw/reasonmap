//! Offline reasoning controls, not an LLM client or a universal capability probe.
//! Exact official endpoint + model ID + protocol matching; unknown means unknown.
//! All bundled data is documentation-backed, not a claim of live-provider tests.
//!
//! ```
//! use reasonmap::{bundled, Protocol};
//! let catalog = bundled().expect("valid bundled data");
//! let model = catalog.resolve("https://api.deepseek.com", Protocol::ChatCompletions,
//!                             "deepseek-flash").expect("documented route");
//! assert_eq!(model.identity.snapshot_version, "DeepSeek-V4.1-Flash");
//! let patch = model.profile.request_patch(Some("high")).expect("known effort");
//! assert_eq!(patch.expect("patch")["reasoning_effort"], "high");
//! ```
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::LazyLock,
};

/// Language-neutral, packaged data: consumers need not depend on this Rust crate.
pub const CATALOG_JSON: &str = include_str!("../data/catalog.json");
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    #[serde(rename = "openai-completions")]
    ChatCompletions,
    #[serde(rename = "openai-responses")]
    Responses,
    #[serde(rename = "anthropic-messages")]
    AnthropicMessages,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Effort {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub request_patch: Map<String, Value>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReasoningProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_effort: Option<String>,
    pub efforts: Vec<Effort>,
}
impl ReasoningProfile {
    /// IDs are opaque. No low->high substitution or unsupported-level fallback.
    pub fn request_patch(
        &self,
        effort: Option<&str>,
    ) -> Result<Option<&Map<String, Value>>, CatalogError> {
        let Some(id) = effort.or(self.default_effort.as_deref()) else {
            return Ok(None);
        };
        self.efforts
            .iter()
            .find(|e| e.id == id)
            .map(|e| Some(&e.request_patch))
            .ok_or_else(|| CatalogError("Unsupported reasoning effort".into()))
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelIdentity {
    /// Information only: the package never rewrites the requested model ID.
    pub canonical_id: String,
    pub display_name: String,
    /// Version observed on checked_on. A rolling alias is not a pinned version.
    pub snapshot_version: String,
    pub legacy_alias: bool,
    /// Input modalities expressible through the matched endpoint's protocol,
    /// from official documentation. Empty means not recorded (unknown); it is
    /// never a claim of text-only input. Recording a modality does not enable
    /// the application to send it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_modalities: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct Compatibility {
    pub family: String,
    pub profile: ReasoningProfile,
    pub identity: ModelIdentity,
    pub catalog_version: String,
    pub checked_on: String,
    pub sources: Vec<String>,
}
#[derive(Debug)]
pub struct Catalog {
    document: CatalogDocument,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogDocument {
    version: String,
    checked_on: String,
    profiles: BTreeMap<String, ReasoningProfile>,
    routes: Vec<Route>,
    identities: BTreeMap<String, ModelIdentity>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    family: String,
    api: Protocol,
    base_urls: Vec<String>,
    models: Vec<String>,
    profile: String,
    sources: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError(String);
impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CatalogError {}
const INPUT_MODALITIES: [&str; 5] = ["text", "image", "audio", "video", "pdf"];
impl Catalog {
    /// Parse and validate before using any external snapshot. Does not fetch it.
    pub fn from_json(json: &str) -> Result<Self, CatalogError> {
        if json.len() > 2 * 1024 * 1024 {
            return Err(CatalogError("Catalog exceeds 2 MiB".into()));
        }
        let document: CatalogDocument = serde_json::from_str(json)
            .map_err(|_| CatalogError("Invalid catalog schema".into()))?;
        let catalog = Self { document };
        catalog.validate()?;
        Ok(catalog)
    }
    pub fn version(&self) -> &str {
        &self.document.version
    }
    pub fn resolve(&self, base: &str, protocol: Protocol, model_id: &str) -> Option<Compatibility> {
        let base = normalized_endpoint(base)?;
        let route = self.document.routes.iter().find(|r| {
            r.api == protocol
                && r.base_urls.contains(&base)
                && r.models.iter().any(|id| id == model_id)
        })?;
        Some(Compatibility {
            family: route.family.clone(),
            profile: self.document.profiles.get(&route.profile)?.clone(),
            identity: self.document.identities.get(model_id)?.clone(),
            catalog_version: self.document.version.clone(),
            checked_on: self.document.checked_on.clone(),
            sources: route.sources.clone(),
        })
    }
    fn validate(&self) -> Result<(), CatalogError> {
        if self.document.version.is_empty()
            || self.document.checked_on.is_empty()
            || self.document.profiles.len() > 512
            || self.document.routes.is_empty()
            || self.document.routes.len() > 512
        {
            return Err(CatalogError("Invalid catalog bounds/version".into()));
        }
        for profile in self.document.profiles.values() {
            let mut ids = BTreeSet::new();
            if profile.efforts.is_empty() || profile.efforts.len() > 64 {
                return Err(CatalogError("Invalid effort count".into()));
            }
            for e in &profile.efforts {
                if e.id.trim().is_empty()
                    || e.id.len() > 128
                    || e.name.trim().is_empty()
                    || !ids.insert(&e.id)
                {
                    return Err(CatalogError("Invalid or duplicate effort".into()));
                }
                if e.request_patch.keys().any(|k| {
                    ![
                        "thinking",
                        "reasoning_effort",
                        "reasoning",
                        "chat_template_kwargs",
                        "output_config",
                        "enable_thinking",
                    ]
                    .contains(&k.as_str())
                }) {
                    return Err(CatalogError(
                        "Patch may only set reasoning extension fields".into(),
                    ));
                }
            }
            if profile
                .default_effort
                .as_ref()
                .is_some_and(|id| !ids.contains(id))
                || serde_json::to_vec(profile)
                    .map_err(|_| CatalogError("Invalid profile".into()))?
                    .len()
                    > 32768
            {
                return Err(CatalogError("Invalid default or oversized profile".into()));
            }
        }
        let mut routes = BTreeSet::new();
        for r in &self.document.routes {
            if !self.document.profiles.contains_key(&r.profile)
                || r.family.is_empty()
                || r.sources.is_empty()
                || r.base_urls.is_empty()
                || r.models.is_empty()
            {
                return Err(CatalogError(
                    "Route lacks profile/identity/provenance".into(),
                ));
            }
            for source in &r.sources {
                if normalized_endpoint(source).is_none() {
                    return Err(CatalogError("Invalid source URL".into()));
                }
            }
            for base in &r.base_urls {
                if normalized_endpoint(base).as_deref() != Some(base.as_str()) {
                    return Err(CatalogError("Invalid route endpoint".into()));
                }
                for id in &r.models {
                    let identity = self
                        .document
                        .identities
                        .get(id)
                        .ok_or_else(|| CatalogError("Missing identity".into()))?;
                    if identity.canonical_id.is_empty()
                        || identity.display_name.is_empty()
                        || identity.snapshot_version.is_empty()
                        || identity.input_modalities.len()
                            != identity
                                .input_modalities
                                .iter()
                                .collect::<BTreeSet<_>>()
                                .len()
                        || identity
                            .input_modalities
                            .iter()
                            .any(|m| !INPUT_MODALITIES.contains(&m.as_str()))
                        || (!identity.input_modalities.is_empty()
                            && !identity.input_modalities.iter().any(|m| m == "text"))
                        || !r.models.contains(&identity.canonical_id)
                        || !routes.insert((base, r.api, id))
                    {
                        return Err(CatalogError("Invalid identity or duplicate route".into()));
                    }
                }
            }
        }
        Ok(())
    }
}
fn normalized_endpoint(base: &str) -> Option<String> {
    let url = url::Url::parse(base).ok()?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.port().is_some()
    {
        return None;
    }
    Some(url.as_str().trim_end_matches('/').to_owned())
}
static BUNDLED: LazyLock<Result<Catalog, CatalogError>> =
    LazyLock::new(|| Catalog::from_json(CATALOG_JSON));
pub fn bundled() -> Result<&'static Catalog, &'static CatalogError> {
    BUNDLED.as_ref()
}
