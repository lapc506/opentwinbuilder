# Proposal

## Change ID: cad-capabilities-onshape

## What

Extend **OpenTwinBuilder** capabilities modeled on Onshape's cloud-native CAD playbook,
so the Studio can behave like a Git-style CAD platform with an AI-driven, code-defined
parametric feature surface:

1. **Git-style CAD version control** — branching, committing, merging, and workspace
   protection for digital-twin/model history (Onshape "Branching & Merging" + "Workspace
   Protection").
2. **FeatureScript-style MCP surface (text-to-code-to-CAD)** — MCP tools that let an AI
   client generate, test, and register reusable, parametric custom features (the Driveshape
   Labs FeatureScript MCP Server pattern: describe → generate code → register reusable tool).
3. **Bill of Process (BOP)** — link Bills of Material (BOM) to process steps / work orders
   so manufacturing automation can drive from parts to operations.

## Motivation

Onshape's model shows that cloud-native CAD is most powerful when:
- **Version control is a first-class, git-like primitive** (branch/merge instead of file
  duplication), and
- **CAD features are code** (FeatureScript) that an LLM can *generate, test, and register*,
  making the MCP server the key AI↔CAD interface.

OpenTwinBuilder already has: a stateless vitemcp MCP server (`ViteMcpServer`), a Thingport
on-demand model source, a Bevy viewport, and a domain/application/ports/infrastructure
(Explicit Architecture) layering. This change adds the versioning, feature-factory, and BOP
capabilities on top, keeping the same layering.

## Scope (in)

- Domain types: `ModelVersion`, `Branch`, `Workspace`, `FeatureDefinition`, `BillOfProcess`.
- MCP tools: `model.branch`, `model.commit`, `model.merge`, `model.history`,
  `workspace.protect`, `feature.generate`, `feature.test`, `feature.register`, `bop.build`,
  `bop.link`.
- Unit tests for each new capability.

## Scope (out)

- Not implementing full 3D geometry kernel / mesh boolean ops (deferred).
- Not implementing a real git remote / collaboration server (local version store only).
- Not implementing Onshape API integration (feature surface is model-specific).

## Success criteria

- `cargo check` passes with new modules.
- MCP server exposes the new tools in `list_tools` and answers `call_tool` for each.
- Main spec is structurally valid (fix the ADDED-header-in-main-spec defect) and the new
  capabilities are present after sync.
