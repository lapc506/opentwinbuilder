# Design — CAD capabilities (versioning, feature factory, BOP)

Keeps the Explicit Architecture layering: `domain` (pure rules) → `application`
(use cases) → `ports` (traits) → `infrastructure` (adapters). No framework types
leak into domain.

## 1. Git-style version control (domain)

New module `domain/versioning.rs`:

- `ModelVersion { id: VersionId, label: String, message: String, parent: Option<VersionId> }`
- `Branch { id: BranchId, name: String, head: VersionId }`
- `Workspace { id: WorkspaceId, name: String, protected: bool }`
- `VersionStore` (in-memory, `Default`) with:
  - `commit(branch, label, message) -> VersionId` (parent = branch head)
  - `create_branch(name, from: VersionId) -> Branch`
  - `merge(from: &Branch, into: &mut Branch) -> VersionId` (linear fast-forward commit)
  - `history(branch) -> Vec<ModelVersion>`
  - `protect(workspace) / is_protected(workspace)`
- Workspace protection rule: a protected workspace rejects direct `commit` and
  requires a `merge` from a branch (review gate).

## 2. FeatureScript-style feature factory (domain)

New module `domain/featurescript.rs`:

- `FeatureDefinition { name: String, code: String, params: Vec<FeatureParam> }`
- `FeatureParam { name: String, kind: ParamKind, default: Option<Value> }`
- `FeatureRegistry` (`Default`) with:
  - `generate(source) -> FeatureDefinition` (stub: captures an LLM-provided code string)
  - `test(def) -> Result<FeatureTestReport, String>` (validates non-empty code, params sane)
  - `register(def) -> Result<(), String>` (dedupe by name; error if name taken)
  - `get(name) -> Option<FeatureDefinition>`

The real LLM generation is out of scope; `generate` accepts a code snippet from the
caller (the AI client) so the MCP tool is the integration surface.

## 3. Bill of Process (domain)

New module `domain/bop.rs`:

- `BillOfProcess { id: BopId, name: String, steps: Vec<ProcessStep> }`
- `ProcessStep { order: u32, operation: String, resource: Option<String> }`
- `BomItem { part_id: String, qty: u32 }`
- `BopBuilder`:
  - `add_part(item)` / `add_step(step)`
  - `link(bom: Vec<BomItem>) -> BillOfProcess` (asserts every part maps to ≥1 step)

## 4. Application layer

`application/studio.rs` extends `Studio` to hold `VersionStore`, `FeatureRegistry`,
and a `BopBuilder` so use cases don't live in the MCP adapter.

## 5. Ports

- `ports/versioning.rs`: `VersionStore` trait (or reuse domain `VersionStore` directly —
  simpler; domain type is pure).
- `ports/featurescript.rs`: `FeatureRegistry` trait.
- `ports/bop.rs`: `BopBuilder` trait.
- `McpServer` port stays; the adapter exposes the new tools.

## 6. Infrastructure: MCP tools

`infrastructure/mcp_server.rs` adds tools (all stateless, return `isError` on failure):

| Tool | Effect |
|------|--------|
| `model.commit` | commit on a branch (label/message) |
| `model.branch` | create branch from a version |
| `model.merge` | fast-forward merge branch → main branch |
| `model.history` | list versions on a branch |
| `workspace.protect` | toggle protection on a workspace |
| `feature.generate` | accept code snippet → FeatureDefinition |
| `feature.test` | validate a definition |
| `feature.register` | register a reusable feature |
| `bop.build` | build a BillOfProcess from parts + steps |
| `bop.link` | assert BOM parts map to steps |

`list_tools` includes all of these (plus existing `capability_report` / `thingport.*`).

## 7. Wiring

`main.rs` composition root constructs `Studio` with the three stores and passes a
reference into `ViteMcpServer`. For simplicity and testability, `ViteMcpServer`
gets an optional `VersionStore` / `FeatureRegistry` / `BopBuilder` (defaults when absent).
