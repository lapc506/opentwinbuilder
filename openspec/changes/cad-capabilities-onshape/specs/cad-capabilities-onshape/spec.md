# CAD Capabilities Spec

## ADDED Requirements

### Requirement: Git-Style CAD Version Control

The toolkit SHALL provide **Git-style version control** for digital-twin/model
history, modeled on Onshape's branching & merging:

- **Commit**: the toolkit SHALL record a new version on a branch (label + message),
  with a parent pointer (linear history).
- **Branch**: the toolkit SHALL create an isolated branch from any version, letting
  experiments proceed without mutating the protected main line.
- **Merge**: the toolkit SHALL merge a branch back into another in a controlled,
  review-gated way (fast-forward commit; no silent direct edits to a protected
  workspace).
- **History**: the toolkit SHALL list the commit history of a branch.
- **Workspace protection**: the toolkit SHALL mark a workspace protected; a protected
  workspace SHALL reject direct commits and SHALL require a merge from a branch.

#### Scenario: branch → merge on a protected workspace

- **WHEN** a user protects a workspace and attempts a direct commit on it
- **THEN** the toolkit rejects the commit (protection active).
- **WHEN** a user creates a branch, commits there, then merges into the protected workspace
- **THEN** the merge succeeds as a controlled fast-forward commit.

### Requirement: FeatureScript-Style MCP Feature Factory

The toolkit SHALL expose an MCP surface that lets AI clients **generate, test, and
register reusable parametric CAD features** (the Onshape Labs FeatureScript MCP
Server pattern: text-to-code-to-CAD).

- **generate**: the toolkit SHALL accept a source code snippet and produce a
  `FeatureDefinition` (name, params, code).
- **test**: the toolkit SHALL validate a definition (non-empty code, well-formed params).
- **register**: the toolkit SHALL register a definition into a reusable registry,
  rejecting duplicate names.
- **reuse**: a registered feature SHALL be retrievable by name for later use.

#### Scenario: text-to-code-to-CAD feature

- **WHEN** an AI client sends a natural-language-derived FeatureScript snippet
- **THEN** the toolkit generates a `FeatureDefinition`, runs `test`, and, on success,
  registers it as a reusable feature.

### Requirement: Bill of Process (BOP)

The toolkit SHALL support **Bill of Process** linking Bills of Material to process
steps for manufacturing automation:

- **BOP build**: the toolkit SHALL assemble a `BillOfProcess` from parts (BOM items)
  and ordered process steps.
- **Link**: the toolkit SHALL assert that every BOM part maps to at least one process step.
- **Serialization**: a built BOP SHALL expose its ordered steps and linked parts.

#### Scenario: BOM → process

- **WHEN** a user provides a BOM and process steps
- **THEN** the toolkit builds a `BillOfProcess` and verifies every part is covered by a step.
