# Tasks — CAD capabilities (versioning, feature factory, BOP)

## Implementation

### Domain

- [ ] D1: Add `domain/versioning.rs` — `VersionId`, `BranchId`, `WorkspaceId`,
      `ModelVersion`, `Branch`, `Workspace`, `VersionStore` (commit/branch/merge/history/protect).
- [ ] D2: Add `domain/featurescript.rs` — `FeatureDefinition`, `FeatureParam`, `ParamKind`,
      `FeatureRegistry` (generate/test/register/get).
- [ ] D3: Add `domain/bop.rs` — `BillOfProcess`, `ProcessStep`, `BomItem`, `BopBuilder`
      (add_part/add_step/link).
- [ ] D4: Wire new domain modules into `domain/mod.rs`.

### Application

- [ ] A1: Extend `application/studio.rs` — hold `VersionStore`, `FeatureRegistry`, `BopBuilder`.

### Infrastructure / MCP

- [ ] I1: Add MCP tools to `ViteMcpServer`: `model.commit`, `model.branch`, `model.merge`,
      `model.history`, `workspace.protect`, `feature.generate`, `feature.test`,
      `feature.register`, `bop.build`, `bop.link`.
- [ ] I2: `list_tools` includes all new tools; `call_tool` dispatches them.
- [ ] I3: Default-initialize the stores when absent (testable standalone).

### Tests

- [ ] T1: Unit tests for `VersionStore` (commit/branch/merge/protect rule).
- [ ] T2: Unit tests for `FeatureRegistry` (generate/test/register, duplicate rejection).
- [ ] T3: Unit tests for `BopBuilder` (build + link assertion).
- [ ] T4: MCP tool dispatch tests (each new tool returns expected JSON; unknown → error).

## Verification

- [ ] `cargo check` passes.
- [ ] `cargo test` (new unit tests) green.
- [ ] `openspec validate cad-capabilities-onshape` valid.
- [ ] Main spec made structurally valid; after archive/sync the new requirements appear.
