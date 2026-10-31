//! MCP server — real `rmcp` (Rust SDK) implementation over the official Model
//! Context Protocol. This is the *provider / inbound* surface: AI clients call
//! the tools (feature factory, versioning, BOP, Thingport). It does NOT contain
//! the A2A orchestrator (that is a separate layer).
//!
//! Protocol: stateless (no session state in the wire sense; stores are owned
//! here and guarded by mutexes). Tools are defined with `#[tool]` macros.

use std::sync::{Arc, Mutex};

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{schemars, tool, tool_router, ErrorData as McpError};

use crate::domain::bop::BopBuilder;
use crate::domain::featurescript::FeatureRegistry;
use crate::domain::versioning::{BranchId, VersionStore, WorkspaceId};
use crate::infrastructure::thingport::ThingportClient;
use crate::ports::model_source::ModelSource;

/// Protocol revision we target (stateless).
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// The Studio MCP server (rmcp). Holds the domain stores guarded by mutexes so
/// `#[tool]` methods (which take `&self`) can mutate them.
#[derive(Debug, Clone)]
pub struct StudioMcpServer {
    pub name: &'static str,
    pub version: &'static str,
    pub thingport: ThingportClient,
    pub versions: Arc<Mutex<VersionStore>>,
    pub features: Arc<Mutex<FeatureRegistry>>,
    pub bop: Arc<Mutex<BopBuilder>>,
}

impl Default for StudioMcpServer {
    fn default() -> Self {
        Self {
            name: "opentwinbuilder",
            version: env!("CARGO_PKG_VERSION"),
            thingport: ThingportClient::new("", ""),
            versions: Arc::new(Mutex::new(VersionStore::new())),
            features: Arc::new(Mutex::new(FeatureRegistry::new())),
            bop: Arc::new(Mutex::new(BopBuilder::new("bop"))),
        }
    }
}

impl StudioMcpServer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thingport(mut self, thingport: ThingportClient) -> Self {
        self.thingport = thingport;
        self
    }

    pub fn with_versions(mut self, versions: VersionStore) -> Self {
        self.versions = Arc::new(Mutex::new(versions));
        self
    }

    pub fn with_features(mut self, features: FeatureRegistry) -> Self {
        self.features = Arc::new(Mutex::new(features));
        self
    }

    pub fn with_bop(mut self, bop: BopBuilder) -> Self {
        self.bop = Arc::new(Mutex::new(bop));
        self
    }
}

// ---- tool params (schemars JSON Schema) ----

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CapabilityParams {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ThingportSearchParams {
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ThingportIdParams {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CommitParams {
    pub branch: String,
    pub label: String,
    pub message: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BranchParams {
    pub name: String,
    #[schemars(description = "base version id (optional)")]
    pub from: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MergeParams {
    pub from: String,
    pub into: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HistoryParams {
    pub branch: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProtectParams {
    pub workspace: String,
    pub r#protected: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FeatureParams {
    pub name: String,
    pub code: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BomItemInput {
    pub part_id: String,
    #[serde(default = "default_qty")]
    pub qty: u32,
}

fn default_qty() -> u32 {
    1
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProcessStepInput {
    pub operation: String,
    pub resource: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BopParams {
    pub name: String,
    #[serde(default)]
    pub parts: Vec<BomItemInput>,
    #[serde(default)]
    pub steps: Vec<ProcessStepInput>,
}

// ---- tool router ----

#[tool_router(server_handler)]
impl StudioMcpServer {
    #[tool(description = "Report the Studio's stateless MCP capabilities")]
    fn capability_report(&self, _: Parameters<CapabilityParams>) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "{{ \"protocol\": \"{}\", \"stateless\": true, \"session_id\": null, \"capabilities\": [\"3d-viewport\",\"entity-picking\",\"rv-odt\",\"opensusd\",\"thingport\",\"versioning\",\"featurescript\",\"bop\"] }}",
            PROTOCOL_VERSION
        ))]))
    }

    #[tool(description = "Search the Thingport model library (no download)")]
    fn thingport_search(&self, Parameters(args): Parameters<ThingportSearchParams>) -> Result<CallToolResult, McpError> {
        if !self.thingport.is_configured() {
            return Err(McpError::invalid_params("thingport not configured (set THINGPORT_URL and THINGPORT_TOKEN)", None));
        }
        match self.thingport.search(&args.query) {
            Ok(items) => {
                let list = items.iter().map(|m| format!("{{\"id\":\"{}\",\"name\":\"{}\"}}", m.id, m.name)).collect::<Vec<_>>().join(",");
                Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"models\":[{}]}}", list))]))
            }
            Err(e) => Err(McpError::internal_error(e, None)),
        }
    }

    #[tool(description = "Fetch one model by id from Thingport (on-demand download)")]
    fn thingport_fetch(&self, Parameters(args): Parameters<ThingportIdParams>) -> Result<CallToolResult, McpError> {
        if !self.thingport.is_configured() {
            return Err(McpError::invalid_params("thingport not configured (set THINGPORT_URL and THINGPORT_TOKEN)", None));
        }
        match self.thingport.fetch(&args.id) {
            Ok(model) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"model\":{},\"imported_to_viewport\":false}}", model))])),
            Err(e) => Err(McpError::internal_error(e, None)),
        }
    }

    #[tool(name = "model.commit", description = "Commit on a branch (Git-style)")]
    fn model_commit(&self, Parameters(args): Parameters<CommitParams>) -> Result<CallToolResult, McpError> {
        let mut store = self.versions.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let branch = BranchId(args.branch.clone());
        if store.head(&branch).is_none() {
            let _ = store.create_branch(args.branch.clone(), None);
        }
        match store.commit(&branch, args.label, args.message, None) {
            Ok(id) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"version\":{}}}", id.0))])),
            Err(e) => Err(McpError::internal_error(format!("{e:?}"), None)),
        }
    }

    #[tool(name = "model.branch", description = "Create a branch from a version")]
    fn model_branch(&self, Parameters(args): Parameters<BranchParams>) -> Result<CallToolResult, McpError> {
        let mut store = self.versions.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let id = store.create_branch(args.name.clone(), args.from.map(crate::domain::versioning::VersionId));
        Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"branch\":\"{}\"}}", id.0))]))
    }

    #[tool(name = "model.merge", description = "Fast-forward merge branch → target")]
    fn model_merge(&self, Parameters(args): Parameters<MergeParams>) -> Result<CallToolResult, McpError> {
        let mut store = self.versions.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        match store.merge(&BranchId(args.from), &BranchId(args.into), None) {
            Ok(id) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"version\":{}}}", id.0))])),
            Err(e) => Err(McpError::internal_error(format!("{e:?}"), None)),
        }
    }

    #[tool(name = "model.history", description = "List commit history of a branch")]
    fn model_history(&self, Parameters(args): Parameters<HistoryParams>) -> Result<CallToolResult, McpError> {
        let store = self.versions.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let h = store.history(&BranchId(args.branch));
        let list = h.iter().map(|v| format!("{{\"id\":{},\"label\":\"{}\",\"message\":\"{}\"}}", v.id.0, v.label, v.message)).collect::<Vec<_>>().join(",");
        Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"history\":[{}]}}", list))]))
    }

    #[tool(name = "workspace.protect", description = "Toggle protection on a workspace")]
    fn workspace_protect(&self, Parameters(args): Parameters<ProtectParams>) -> Result<CallToolResult, McpError> {
        let mut store = self.versions.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let ws = WorkspaceId(args.workspace.clone());
        let _ = store.add_workspace(args.workspace.clone(), false);
        match store.protect(&ws, args.r#protected) {
            Ok(()) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"workspace\":\"{}\",\"protected\":{}}}", ws.0, args.r#protected))])),
            Err(e) => Err(McpError::internal_error(format!("{e:?}"), None)),
        }
    }

    #[tool(name = "feature.generate", description = "Generate a reusable feature definition from code")]
    fn feature_generate(&self, Parameters(args): Parameters<FeatureParams>) -> Result<CallToolResult, McpError> {
        let reg = self.features.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let def = reg.generate(args.name.clone(), args.code);
        Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"feature\":{{\"name\":\"{}\",\"params\":{}}}}}", def.name, def.params.len()))]))
    }

    #[tool(name = "feature.test", description = "Validate a feature definition")]
    fn feature_test(&self, Parameters(args): Parameters<FeatureParams>) -> Result<CallToolResult, McpError> {
        let reg = self.features.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let def = reg.generate(args.name.clone(), args.code);
        let report = reg.test(&def);
        Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"ok\":{},\"issues\":{:?}}}", report.ok, report.issues))]))
    }

    #[tool(name = "feature.register", description = "Register a reusable feature")]
    fn feature_register(&self, Parameters(args): Parameters<FeatureParams>) -> Result<CallToolResult, McpError> {
        let mut reg = self.features.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let def = reg.generate(args.name.clone(), args.code);
        match reg.register(def) {
            Ok(()) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"registered\":\"{}\"}}", args.name))])),
            Err(e) => Err(McpError::internal_error(format!("{e:?}"), None)),
        }
    }

    #[tool(name = "bop.build", description = "Build a Bill of Process from parts + steps")]
    fn bop_build(&self, Parameters(args): Parameters<BopParams>) -> Result<CallToolResult, McpError> {
        let mut builder = BopBuilder::new(args.name.clone());
        for p in &args.parts {
            builder.add_part(p.part_id.clone(), p.qty);
        }
        for (i, s) in args.steps.iter().enumerate() {
            builder.add_step(i as u32 + 1, s.operation.clone(), s.resource.clone());
        }
        match builder.build() {
            Ok(bop) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "{{\"name\":\"{}\",\"parts\":{},\"steps\":{}}}", bop.name, bop.parts.len(), bop.steps.len()
            ))])),
            Err(e) => Err(McpError::internal_error(format!("{e:?}"), None)),
        }
    }

    #[tool(name = "bop.link", description = "Assert BOM parts map to process steps")]
    fn bop_link(&self, Parameters(args): Parameters<BopParams>) -> Result<CallToolResult, McpError> {
        let mut builder = BopBuilder::new(args.name.clone());
        for p in &args.parts {
            builder.add_part(p.part_id.clone(), p.qty);
        }
        for (i, s) in args.steps.iter().enumerate() {
            builder.add_step(i as u32 + 1, s.operation.clone(), s.resource.clone());
        }
        let linked = builder.link();
        Ok(CallToolResult::success(vec![ContentBlock::text(format!("{{\"linked_parts\":{:?}}}", linked))]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::ClientHandler as _;
    use rmcp::ServerHandler;
    use rmcp::model::{CallToolRequestParams, CallToolResult};
    use rmcp::service::{RunningService, ServiceError};
    use rmcp::{RoleClient, ServiceExt};

    /// A live MCP client wired to a real `StudioMcpServer` over an in-memory
    /// duplex transport. This exercises the actual `#[tool_router]` dispatch
    /// path (`tools/call`), not just the inner domain logic.
    type TestClient = RunningService<RoleClient, ()>;

    /// Given a server handler, spawn it on a duplex transport and connect a
    /// client. Returns (server handle, client).
    async fn start_server_and_client(
        server: StudioMcpServer,
    ) -> (tokio::task::JoinHandle<anyhow::Result<()>>, TestClient) {
        let (server_transport, client_transport) = tokio::io::duplex(4096);
        let srv = tokio::spawn(async move {
            let service = server.serve(server_transport).await?;
            service.waiting().await?;
            anyhow::Ok(())
        });
        let client = ().serve(client_transport).await.expect("client connects to server");
        (srv, client)
    }

    async fn cleanup(srv: tokio::task::JoinHandle<anyhow::Result<()>>, client: TestClient) {
        let _ = client.cancel().await;
        srv.abort();
    }

    async fn call_tool(
        client: &TestClient,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CallToolResult, ServiceError> {
        client
            .call_tool(
                CallToolRequestParams::new(name.to_string())
                    .with_arguments(args.as_object().cloned().unwrap_or_default()),
            )
            .await
    }

    fn json_text(result: &CallToolResult) -> serde_json::Value {
        let text = result.content[0].as_text().expect("tool returned text content");
        serde_json::from_str(&text.text).expect("response is valid JSON")
    }

    #[test]
    fn server_constructs_and_reports_info() {
        let s = StudioMcpServer::new();
        // rmcp handler produces an initialize result (protocol + capabilities) —
        // constructor + get_info must not panic.
        let _info = s.get_info();
    }

    #[tokio::test]
    async fn model_commit_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client calls `model.commit` on a brand-new branch.
        let result = call_tool(
            &client,
            "model.commit",
            serde_json::json!({"branch": "main", "label": "v1", "message": "init"}),
        )
        .await
        .expect("model.commit dispatches successfully");

        // Then the dispatch returns the expected JSON envelope.
        let json = json_text(&result);
        assert_eq!(json["version"], 1);

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn model_branch_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client calls `model.branch`.
        let result = call_tool(&client, "model.branch", serde_json::json!({"name": "feat"}))
            .await
            .expect("model.branch dispatches successfully");

        // Then the dispatch returns the created branch id.
        let json = json_text(&result);
        assert_eq!(json["branch"], "feat");

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn model_merge_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When we set up two branches and merge one into the other.
        call_tool(&client, "model.branch", serde_json::json!({"name": "main"}))
            .await
            .unwrap();
        call_tool(
            &client,
            "model.commit",
            serde_json::json!({"branch": "main", "label": "v1", "message": "init"}),
        )
        .await
        .unwrap();
        call_tool(&client, "model.branch", serde_json::json!({"name": "feat", "from": 1}))
            .await
            .unwrap();
        call_tool(
            &client,
            "model.commit",
            serde_json::json!({"branch": "feat", "label": "f1", "message": "exp"}),
        )
        .await
        .unwrap();

        let result = call_tool(
            &client,
            "model.merge",
            serde_json::json!({"from": "feat", "into": "main"}),
        )
        .await
        .expect("model.merge dispatches successfully");

        // Then the dispatch returns a new (merge) version.
        let json = json_text(&result);
        assert_eq!(json["version"], 3);

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn model_history_dispatches_to_expected_json() {
        // Given a fresh server handler with two commits on `main`.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;
        call_tool(&client, "model.branch", serde_json::json!({"name": "main"}))
            .await
            .unwrap();
        call_tool(
            &client,
            "model.commit",
            serde_json::json!({"branch": "main", "label": "v1", "message": "init"}),
        )
        .await
        .unwrap();
        call_tool(
            &client,
            "model.commit",
            serde_json::json!({"branch": "main", "label": "v2", "message": "work"}),
        )
        .await
        .unwrap();

        // When a client requests the history for `main`.
        let result = call_tool(&client, "model.history", serde_json::json!({"branch": "main"}))
            .await
            .expect("model.history dispatches successfully");

        // Then the dispatch returns the commit list, newest first.
        let json = json_text(&result);
        let history = json["history"].as_array().expect("history is an array");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0]["id"], 2);
        assert_eq!(history[1]["id"], 1);

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn workspace_protect_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client toggles protection on a workspace.
        let result = call_tool(
            &client,
            "workspace.protect",
            serde_json::json!({"workspace": "prod", "protected": true}),
        )
        .await
        .expect("workspace.protect dispatches successfully");

        // Then the dispatch returns the workspace protection state.
        let json = json_text(&result);
        assert_eq!(json["workspace"], "prod");
        assert_eq!(json["protected"], true);

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn feature_generate_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client generates a feature definition.
        let result = call_tool(
            &client,
            "feature.generate",
            serde_json::json!({"name": "sprocket", "code": "feature sprocket"}),
        )
        .await
        .expect("feature.generate dispatches successfully");

        // Then the dispatch returns the generated feature metadata.
        let json = json_text(&result);
        assert_eq!(json["feature"]["name"], "sprocket");
        assert_eq!(json["feature"]["params"], 0);

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn feature_test_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client tests a valid feature definition.
        let result = call_tool(
            &client,
            "feature.test",
            serde_json::json!({"name": "ok_feature", "code": "valid code"}),
        )
        .await
        .expect("feature.test dispatches successfully");

        // Then the dispatch returns a passing validation report.
        let json = json_text(&result);
        assert_eq!(json["ok"], true);
        assert_eq!(json["issues"], serde_json::json!([]));

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn feature_register_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client registers a feature.
        let result = call_tool(
            &client,
            "feature.register",
            serde_json::json!({"name": "gear", "code": "feature gear"}),
        )
        .await
        .expect("feature.register dispatches successfully");

        // Then the dispatch returns the registered feature name.
        let json = json_text(&result);
        assert_eq!(json["registered"], "gear");

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn bop_build_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client builds a Bill of Process from parts + steps.
        let result = call_tool(
            &client,
            "bop.build",
            serde_json::json!({
                "name": "line-a",
                "parts": [
                    {"part_id": "P-001", "qty": 2},
                    {"part_id": "P-002", "qty": 1}
                ],
                "steps": [
                    {"operation": "feed"},
                    {"operation": "assemble", "resource": "robot-1"}
                ]
            }),
        )
        .await
        .expect("bop.build dispatches successfully");

        // Then the dispatch returns counts of the built BOP.
        let json = json_text(&result);
        assert_eq!(json["name"], "line-a");
        assert_eq!(json["parts"], 2);
        assert_eq!(json["steps"], 2);

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn bop_link_dispatches_to_expected_json() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client asserts BOM→step linkage.
        let result = call_tool(
            &client,
            "bop.link",
            serde_json::json!({
                "name": "line-a",
                "parts": [
                    {"part_id": "P-001", "qty": 2},
                    {"part_id": "P-002", "qty": 1}
                ],
                "steps": [
                    {"operation": "feed"},
                    {"operation": "assemble", "resource": "robot-1"}
                ]
            }),
        )
        .await
        .expect("bop.link dispatches successfully");

        // Then the dispatch returns the linked part set.
        let json = json_text(&result);
        assert_eq!(json["linked_parts"], serde_json::json!(["P-001", "P-002"]));

        cleanup(srv, client).await;
    }

    #[tokio::test]
    async fn unknown_tool_returns_error() {
        // Given a fresh server handler.
        let (srv, client) = start_server_and_client(StudioMcpServer::new()).await;

        // When a client calls a tool that does not exist.
        let result = call_tool(&client, "no.such.tool", serde_json::json!({})).await;

        // Then the dispatch fails with a protocol error (not a silent success).
        assert!(result.is_err(), "unknown tool must return an error");

        cleanup(srv, client).await;
    }
}
