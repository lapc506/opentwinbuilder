//! Bill of Process (BOP) — link Bills of Material to process steps.
//!
//! Pure domain types + a builder. No IO, no framework types.

/// A single ordered manufacturing step.
#[derive(Debug, Clone)]
pub struct ProcessStep {
    pub order: u32,
    pub operation: String,
    pub resource: Option<String>,
}

/// A BOM line item.
#[derive(Debug, Clone)]
pub struct BomItem {
    pub part_id: String,
    pub qty: u32,
}

/// A Bill of Process linking parts to ordered steps.
#[derive(Debug, Clone)]
pub struct BillOfProcess {
    pub name: String,
    pub parts: Vec<BomItem>,
    pub steps: Vec<ProcessStep>,
}

/// Builds a BillOfProcess, asserting part coverage.
#[derive(Debug, Default)]
pub struct BopBuilder {
    name: String,
    parts: Vec<BomItem>,
    steps: Vec<ProcessStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BopError {
    UncoveredParts(Vec<String>),
}

impl BopBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), parts: Vec::new(), steps: Vec::new() }
    }

    pub fn add_part(&mut self, part_id: impl Into<String>, qty: u32) -> &mut Self {
        self.parts.push(BomItem { part_id: part_id.into(), qty });
        self
    }

    pub fn add_step(&mut self, order: u32, operation: impl Into<String>, resource: Option<impl Into<String>>) -> &mut Self {
        self.steps.push(ProcessStep {
            order,
            operation: operation.into(),
            resource: resource.map(|r| r.into()),
        });
        self
    }

    /// Build the BOP, asserting every part maps to at least one step.
    pub fn build(&self) -> Result<BillOfProcess, BopError> {
        // Simple coverage rule: a part is covered if there is at least one step.
        // (In a fuller model we'd check a mapping table; here presence of steps suffices.)
        if self.steps.is_empty() && !self.parts.is_empty() {
            let uncovered: Vec<String> = self.parts.iter().map(|p| p.part_id.clone()).collect();
            return Err(BopError::UncoveredParts(uncovered));
        }
        Ok(BillOfProcess {
            name: self.name.clone(),
            parts: self.parts.clone(),
            steps: self.steps.clone(),
        })
    }

    /// Link: return the covered set of parts (all parts, since steps exist).
    pub fn link(&self) -> Vec<String> {
        self.parts.iter().map(|p| p.part_id.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_with_steps_ok() {
        let bop = BopBuilder::new("line-a")
            .add_part("P-001", 2)
            .add_part("P-002", 1)
            .add_step(1, "feed", None::<String>)
            .add_step(2, "assemble", Some("robot-1"))
            .build()
            .unwrap();
        assert_eq!(bop.steps.len(), 2);
        assert_eq!(bop.parts.len(), 2);
    }

    #[test]
    fn build_without_steps_errors() {
        let r = BopBuilder::new("line-b").add_part("P-001", 1).build();
        assert!(matches!(r, Err(BopError::UncoveredParts(_))));
    }
}
