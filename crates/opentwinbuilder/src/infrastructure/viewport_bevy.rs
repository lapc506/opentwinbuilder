//! Bevy 3D viewport — infrastructure adapter of the `Viewport` port.
//!
//! This is the only module that touches Bevy. It renders the scene and feeds
//! the shared `SelectionHandle` (the port) which the Iced UI and application
//! layer read. Entity Picking (`bevy_picking` + `mesh_picking`) drives hover,
//! and a left-click commits the selection.

use bevy::picking::events::{Pointer, PointerClick};
use bevy::picking::hover::HoverMap;
use bevy::picking::Pickable;
use bevy::prelude::*;

use crate::domain::twin::{Selection, TwinId};
use crate::ports::viewport::{ImportHandle, SelectionHandle};

/// Component tagging a "digital twin" entity in the viewport.
#[derive(Component, Debug, Clone)]
pub struct TwinMarker {
    pub id: TwinId,
}

/// Bevy plugin that wires the 3D viewport to the shared selection + import state.
pub struct StudioViewportPlugin {
    pub handle: SelectionHandle,
    pub imports: ImportHandle,
}

impl Plugin for StudioViewportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.handle.clone())
            .insert_resource(self.imports.clone())
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (pick_system, process_imports));
    }
}

/// Minimal but real scene: ground + a few pickable twin cubes.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 4.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((DirectionalLight::default(), Transform::from_xyz(4., 8., 4.)));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.25, 0.3),
            ..default()
        })),
        Transform::from_xyz(0., 0., 0.),
    ));

    for (idx, offset) in [
        Vec3::new(-1.5, 0.5, 0.0),
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.5, 0.5, 0.0),
    ]
    .iter()
    .enumerate()
    {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.4 + idx as f32 * 0.15, 0.5, 0.7),
                ..default()
            })),
            Transform::from_translation(*offset),
            TwinMarker {
                id: TwinId(format!("twin-{}", idx + 1)),
            },
            Pickable::default(),
        ));
    }
}

/// Hover → live preview; left-click → commit selection into the shared handle.
fn pick_system(
    mut clicks: MessageReader<PointerClick>,
    handle: Res<SelectionHandle>,
    hover_map: Res<HoverMap>,
    twins: Query<(&TwinMarker, &Pickable)>,
) {
    // Hover updates a transient selection that is not "committed".
    let mut hovered: Option<TwinId> = None;
    for (_ptr, map) in hover_map.iter() {
        if let Some((entity, _)) = map.iter().next() {
            if let Ok((marker, _)) = twins.get(*entity) {
                hovered = Some(marker.id.clone());
            }
        }
    }

    // A left click commits the hovered twin as the active selection.
    for click in clicks.read() {
        if click.button == PointerButton::Primary && hovered.is_some() {
            handle.set(Selection { picked: hovered.clone() });
        }
    }
}

/// Drain the inbound `ImportHandle` and spawn a scene entity per fetched model.
fn process_imports(
    mut commands: Commands,
    imports: Res<ImportHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let batch = match imports.take_all() {
        Ok(b) => b,
        Err(e) => {
            error!("viewport import drain failed: {e}");
            return;
        }
    };

    for (i, req) in batch.into_iter().enumerate() {
        let mesh = mesh_for_model(&req.data);
        let size = mesh_size_hint(&req.data);
        // Place imports in a row beside the starter twins, on the ground plane.
        let x = 3.0 + i as f32 * 1.5;

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.55, 0.2),
                ..default()
            })),
            Transform::from_translation(Vec3::new(x, size * 0.5, 0.0)),
            TwinMarker {
                id: TwinId(format!("thingport-{}", req.model_id)),
            },
            Pickable::default(),
        ));
    }
}

/// Map a fetched model payload to a scene mesh. The Thingport JSON schema is not
/// fully specified, so we honour a couple of well-known shape hints and fall back
/// to a distinctive cube so an imported model is always visible.
fn mesh_for_model(data: &serde_json::Value) -> Mesh {
    let shape = data.get("shape").and_then(|v| v.as_str()).unwrap_or("cube");
    let size = mesh_size_hint(data);
    match shape {
        "plane" => Plane3d::default().mesh().size(size, size).into(),
        _ => Cuboid::new(size, size, size).mesh().into(),
    }
}

/// Optional `size` hint from the model payload (defaults to 1.0).
fn mesh_size_hint(data: &serde_json::Value) -> f32 {
    data.get("size")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0)
}
