use bevy::prelude::*;
use space_editor::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Local;

fn despawn_local(mut commands: Commands, q: Query<Entity, With<Local>>) {
    for e in q.iter() {
        commands.get_entity(e).unwrap().despawn_recursive();
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(EditorState::Game), despawn_local);
    }
}
