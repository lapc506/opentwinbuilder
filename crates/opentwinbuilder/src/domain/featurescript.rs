//! FeatureScript-style feature factory (text-to-code-to-CAD).
//!
//! Pure domain types + an in-memory registry. The MCP tool is the integration
//! surface: an AI client supplies a source snippet; we wrap it in a reusable,
//! parametric `FeatureDefinition`.

use serde_json::Value;
use std::collections::BTreeMap;

/// Parameter kind for a reusable feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamKind {
    Length,
    Angle,
    Integer,
    Choice,
}

/// A single parameter of a feature.
#[derive(Debug, Clone)]
pub struct FeatureParam {
    pub name: String,
    pub kind: ParamKind,
    pub default: Option<Value>,
}

/// A reusable, parametric CAD feature definition.
#[derive(Debug, Clone)]
pub struct FeatureDefinition {
    pub name: String,
    pub code: String,
    pub params: Vec<FeatureParam>,
}

/// Result of testing a definition.
#[derive(Debug, Clone)]
pub struct FeatureTestReport {
    pub name: String,
    pub ok: bool,
    pub issues: Vec<String>,
}

/// In-memory feature registry.
#[derive(Debug, Default)]
pub struct FeatureRegistry {
    features: BTreeMap<String, FeatureDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureError {
    DuplicateName,
    NotFound,
}

impl FeatureRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a definition from a caller-supplied code snippet.
    pub fn generate(&self, name: impl Into<String>, code: impl Into<String>) -> FeatureDefinition {
        FeatureDefinition {
            name: name.into(),
            code: code.into(),
            params: Vec::new(),
        }
    }

    /// Validate a definition (non-empty code; params well-formed).
    pub fn test(&self, def: &FeatureDefinition) -> FeatureTestReport {
        let mut issues = Vec::new();
        if def.code.trim().is_empty() {
            issues.push("code is empty".into());
        }
        if def.name.trim().is_empty() {
            issues.push("name is empty".into());
        }
        for p in &def.params {
            if p.name.trim().is_empty() {
                issues.push("param name is empty".into());
            }
        }
        FeatureTestReport {
            name: def.name.clone(),
            ok: issues.is_empty(),
            issues,
        }
    }

    /// Register a definition (dedupe by name).
    pub fn register(&mut self, def: FeatureDefinition) -> Result<(), FeatureError> {
        if self.features.contains_key(&def.name) {
            return Err(FeatureError::DuplicateName);
        }
        self.features.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<FeatureDefinition, FeatureError> {
        self.features.get(name).cloned().ok_or(FeatureError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_test_register_reuse() {
        let mut reg = FeatureRegistry::new();
        let def = reg.generate("sprocket", "feature sprocket ...");
        assert!(reg.test(&def).ok);
        reg.register(def).unwrap();
        let got = reg.get("sprocket").unwrap();
        assert_eq!(got.name, "sprocket");
    }

    #[test]
    fn duplicate_rejected() {
        let mut reg = FeatureRegistry::new();
        reg.register(reg.generate("x", "code")).unwrap();
        let r = reg.register(reg.generate("x", "other"));
        assert_eq!(r, Err(FeatureError::DuplicateName));
    }

    #[test]
    fn empty_code_fails_test() {
        let reg = FeatureRegistry::new();
        let def = reg.generate("bad", "   ");
        assert!(!reg.test(&def).ok);
    }
}
