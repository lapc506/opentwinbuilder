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

#[cfg(feature = "iced-shell")]
mod iced_shell {
    use iced::Element;

    use crate::infrastructure::ribbon::{
        ribbon_view, RibbonCommand, RibbonGroupSpec, RibbonMessage, RibbonState,
    };

    #[derive(Debug, Clone, Default)]
    pub struct RibbonDemo {
        ribbon: RibbonState,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Ribbon(RibbonMessage),
    }

    fn update(state: &mut RibbonDemo, message: Message) {
        let Message::Ribbon(m) = message;
        match m {
            RibbonMessage::ToggleCollapse => state.ribbon.toggle_collapse(),
            RibbonMessage::SelectTab(i) => state.ribbon.select_tab(i),
            RibbonMessage::Command(_) => {}
        }
    }

    fn view(state: &RibbonDemo) -> Element<Message> {
        use std::sync::LazyLock;
        static TABS: LazyLock<Vec<&'static str>> =
            LazyLock::new(|| vec!["Home", "Editor", "Viewport", "Thingport", "Estado"]);
        static GROUPS: LazyLock<Vec<RibbonGroupSpec>> = LazyLock::new(|| {
            vec![
                RibbonGroupSpec {
                    title: "Selección",
                    commands: vec![
                        RibbonCommand { label: "Todo", tip: "" },
                        RibbonCommand { label: "Limpiar", tip: "" },
                    ],
                },
                RibbonGroupSpec {
                    title: "Thingport",
                    commands: vec![
                        RibbonCommand { label: "Buscar", tip: "" },
                        RibbonCommand { label: "Importar", tip: "" },
                    ],
                },
                RibbonGroupSpec {
                    title: "Archivo",
                    commands: vec![RibbonCommand { label: "Guardar", tip: "" }],
                },
            ]
        });
        ribbon_view(&TABS, &state.ribbon, &GROUPS).map(Message::Ribbon)
    }

    pub fn run() -> iced::Result {
        // Force Vulkan as the wgpu backend (primary). Metal (Apple) and other
        // backends are deferred; WGPU_BACKEND=vulkan guarantees the NVIDIA/Intel
        // path on this host.
        std::env::set_var("WGPU_BACKEND", "vulkan");
        iced::run("OpenTwinBuilder — Ribbon (iced)", update, view)
    }
}

use bevy::prelude::*;

use application::studio::Studio;
use infrastructure::mcp_server::{ViteMcpServer, PROTOCOL_VERSION};
use infrastructure::thingport::ThingportClient;
use infrastructure::viewport_bevy::StudioViewportPlugin;
use ports::mcp::McpServer;
use ports::viewport::SelectionHandle;

fn main() {
    #[cfg(feature = "iced-shell")]
    {
        let _ = iced_shell::run();
        std::process::exit(0);
    }

    // --- Composition root ---
    let selection = SelectionHandle::default();
    let studio = Studio::new(selection.clone());
    let _ = studio;

    // Stateless vitemcp MCP server boots next to the app (no session state).
    // Thingport on-demand model source: THINGPORT_URL / THINGPORT_TOKEN env.
    let thingport = ThingportClient::new(
        std::env::var("THINGPORT_URL").unwrap_or_default(),
        std::env::var("THINGPORT_TOKEN").unwrap_or_default(),
    );
    let mcp = ViteMcpServer::new().with_thingport(thingport);
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
