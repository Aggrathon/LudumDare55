use bevy::audio::Volume;
use bevy::prelude::*;
use space_editor::prelude::*;

use crate::level::Gameplay;

#[derive(Clone, Copy, Reflect, Default)]
#[reflect(Default)]
pub enum AudioLibrary {
    #[default]
    Arrow,
}

impl AudioLibrary {
    pub fn path(&self) -> &'static str {
        match self {
            AudioLibrary::Arrow => {
                if fastrand::bool() {
                    "audio/shoot_bow_01.ogg"
                } else {
                    "audio/shoot_bow_02.ogg"
                }
            }
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct PlayOnAwake {
    sound: AudioLibrary,
    volume: Volume,
    pitch: f32,
}

impl Default for PlayOnAwake {
    fn default() -> Self {
        Self {
            sound: Default::default(),
            volume: Volume::new(0.5),
            pitch: 0.2,
        }
    }
}

fn play_on_awake(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q: Query<(Entity, &PlayOnAwake)>,
) {
    for (entity, play) in q.iter() {
        let settings = PlaybackSettings::ONCE
            .with_speed(fastrand::f32() * play.pitch * 2.0 - play.pitch + 1.0)
            .with_volume(play.volume);
        commands
            .get_entity(entity)
            .unwrap()
            .insert(AudioBundle {
                source: asset_server.load(play.sound.path()),
                settings,
            })
            .remove::<PlayOnAwake>();
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AudioLibrary>()
            .editor_registry::<PlayOnAwake>()
            .add_systems(PreUpdate, play_on_awake.in_set(Gameplay));
    }
}
