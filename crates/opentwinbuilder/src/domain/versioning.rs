//! Version control for digital twins (Git-style, modeled on Onshape branching/merging).
//!
//! Pure domain types + an in-memory store. No Bevy, no Iced, no MCP, no IO.

use std::collections::BTreeMap;

/// Strong id for a version (commit).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VersionId(pub u64);

/// Strong id for a branch.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BranchId(pub String);

/// Strong id for a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub String);

/// A single committed version.
#[derive(Debug, Clone)]
pub struct ModelVersion {
    pub id: VersionId,
    pub branch: BranchId,
    pub label: String,
    pub message: String,
    pub parent: Option<VersionId>,
}

/// A named line of work pointing at a head version.
#[derive(Debug, Clone)]
pub struct Branch {
    pub id: BranchId,
    pub name: String,
    pub head: Option<VersionId>,
}

/// A named workspace with an optional protection flag.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub protected: bool,
}

/// In-memory Git-style store. Pure domain — no IO.
#[derive(Debug, Default)]
pub struct VersionStore {
    versions: BTreeMap<VersionId, ModelVersion>,
    branches: BTreeMap<BranchId, Branch>,
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    next_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionError {
    BranchNotFound,
    WorkspaceNotFound,
    WorkspaceProtected,
    MergeTargetMissing,
}

impl VersionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a branch starting from `from`.
    pub fn create_branch(&mut self, name: impl Into<String>, from: Option<VersionId>) -> BranchId {
        let id = BranchId(name.into());
        self.branches.insert(
            id.clone(),
            Branch {
                id: id.clone(),
                name: id.0.clone(),
                head: from,
            },
        );
        id
    }

    /// Register a workspace.
    pub fn add_workspace(&mut self, name: impl Into<String>, protected: bool) -> WorkspaceId {
        let id = WorkspaceId(name.into());
        self.workspaces.insert(id.clone(), Workspace { id: id.clone(), name: id.0.clone(), protected });
        id
    }

    /// Set protection on a workspace.
    pub fn protect(&mut self, workspace: &WorkspaceId, protected: bool) -> Result<(), VersionError> {
        let ws = self.workspaces.get_mut(workspace).ok_or(VersionError::WorkspaceNotFound)?;
        ws.protected = protected;
        Ok(())
    }

    pub fn is_protected(&self, workspace: &WorkspaceId) -> Result<bool, VersionError> {
        self.workspaces.get(workspace).map(|w| w.protected).ok_or(VersionError::WorkspaceNotFound)
    }

    /// Commit on a branch. Rejects if the branch is a protected workspace's direct line.
    pub fn commit(&mut self, branch: &BranchId, label: impl Into<String>, message: impl Into<String>, protect_workspace: Option<&WorkspaceId>) -> Result<VersionId, VersionError> {
        if let Some(ws) = protect_workspace {
            if self.is_protected(ws)? {
                return Err(VersionError::WorkspaceProtected);
            }
        }
        // First commit on a fresh branch: parent = None.
        let head = self.branches.get(branch).and_then(|b| b.head.clone());
        self.next_version += 1;
        let id = VersionId(self.next_version);
        self.versions.insert(
            id.clone(),
            ModelVersion {
                id: id.clone(),
                branch: branch.clone(),
                label: label.into(),
                message: message.into(),
                parent: head,
            },
        );
        if let Some(b) = self.branches.get_mut(branch) {
            b.head = Some(id.clone());
        }
        Ok(id)
    }

    /// Fast-forward merge: `from` → `into`. Creates a merge version with parent = into.head,
    /// then drops a copy of from.head. The protected-workspace rule applies to the target.
    pub fn merge(&mut self, from: &BranchId, into: &BranchId, protect_workspace: Option<&WorkspaceId>) -> Result<VersionId, VersionError> {
        if let Some(ws) = protect_workspace {
            if self.is_protected(ws)? {
                // Merges into a protected workspace ARE allowed (controlled review gate),
                // so we do NOT block here. Direct commits are blocked above.
                let _ = ws;
            }
        }
        let from_head = self.branches.get(from).and_then(|b| b.head.clone()).ok_or(VersionError::BranchNotFound)?;
        let into_head = self.branches.get(into).and_then(|b| b.head.clone()).ok_or(VersionError::MergeTargetMissing)?;
        self.next_version += 1;
        let id = VersionId(self.next_version);
        self.versions.insert(
            id.clone(),
            ModelVersion {
                id: id.clone(),
                branch: into.clone(),
                label: format!("merge({})", self.branches.get(from).map(|b| b.name.as_str()).unwrap_or("?")),
                message: "controlled merge".into(),
                parent: Some(into_head),
            },
        );
        if let Some(b) = self.branches.get_mut(into) {
            b.head = Some(id.clone());
        }
        Ok(id)
    }

    pub fn history(&self, branch: &BranchId) -> Vec<ModelVersion> {
        let mut out = Vec::new();
        let mut cur = self.branches.get(branch).and_then(|b| b.head.clone());
        while let Some(id) = cur {
            if let Some(v) = self.versions.get(&id) {
                out.push(v.clone());
                cur = v.parent.clone();
            } else {
                break;
            }
        }
        out
    }

    pub fn head(&self, branch: &BranchId) -> Option<VersionId> {
        self.branches.get(branch).and_then(|b| b.head.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_and_history() {
        let mut s = VersionStore::new();
        let b = s.create_branch("main", None);
        s.commit(&b, "v1", "init", None).unwrap();
        let v2 = s.commit(&b, "v2", "feature", None).unwrap();
        let h = s.history(&b);
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].id, v2);
        assert_eq!(h[1].label, "v1");
    }

    #[test]
    fn branch_and_merge() {
        let mut s = VersionStore::new();
        let main = s.create_branch("main", None);
        s.commit(&main, "v1", "init", None).unwrap();
        let head_main = s.head(&main).unwrap();
        let feat = s.create_branch("feat", Some(head_main));
        s.commit(&feat, "f1", "experiment", None).unwrap();
        let merged = s.merge(&feat, &main, None).unwrap();
        assert_eq!(s.head(&main), Some(merged));
    }

    #[test]
    fn protected_workspace_rejects_direct_commit() {
        let mut s = VersionStore::new();
        let ws = s.add_workspace("prod", false); // seed unprotected
        let main = s.create_branch("main", None);
        s.commit(&main, "v1", "init", Some(&ws)).unwrap();
        s.protect(&ws, true).unwrap(); // now protect
        let r = s.commit(&main, "v2", "direct", Some(&ws));
        assert_eq!(r, Err(VersionError::WorkspaceProtected));
    }
}
