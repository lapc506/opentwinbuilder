//! OpenTwinBuilder — Digital Twin Studio (Explicit Architecture monorepo).
//!
//! Layering (Herberto Graca — Explicit Architecture):
//!   domain/          pure rules (twin model, newtypes)
//!   application/     use cases (Studio service)
//!   ports/           interfaces (McpServer, Viewport)
//!   infrastructure/  adapters (Bevy viewport, vitemcp MCP server)
//!
//! Composition root: builds the concrete adapters and starts the Bevy app.

mod application;
mod domain;
mod infrastructure;
mod ports;

use bevy::prelude::*;

use application::studio::Studio;
use infrastructure::mcp_server::{ViteMcpServer, PROTOCOL_VERSION};
use infrastructure::viewport_bevy::StudioViewportPlugin;
use ports::mcp::McpServer;
use ports::viewport::SelectionHandle;

fn main() {
    // --- Composition root ---
    let selection = SelectionHandle::default();
    let studio = Studio::new(selection.clone());

    // Stateless vitemcp MCP server boots next to the app (no session state).
    let mcp = ViteMcpServer::new();
    let _ = mcp.list_tools();
    println!("OpenTwinBuilder MCP protocol: {}", PROTOCOL_VERSION);

    // Bevy app: 3D viewport adapter → shared selection.
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenTwinBuilder — Digital Twin Studio".into(),
                resolution: (1280u32, 800u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(StudioViewportPlugin { handle: selection })
        .run();
}
