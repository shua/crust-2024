use std::collections::HashMap as Map;
use std::f32::consts::PI;

use bevy::{
    app::AppExit,
    input::mouse::MouseWheel,
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    window::PrimaryWindow,
};

#[derive(Component)]
pub struct Control;
#[derive(Component, Clone, Copy, Default, Debug)]
pub enum Collide {
    #[default]
    Square,
    StepL,
    StepR,
    SlopeL,
    SlopeR,
}
#[derive(Resource)]
pub struct PhysicsTick(pub f32);
#[derive(Component, Default)]
pub struct Movement {
    ctl: Vec2,
    force: Vec2,
    out: Vec2,
    climb: bool,
}
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug)]
pub struct Tile(u8);
#[derive(Event)]
pub struct Quit; // custom quit event used to save map before actual AppExit
#[derive(Resource, Default, Deref)]
pub struct TileTypes(pub Vec<(Color, Collide, Option<(Handle<Image>, (f32, f32), f32)>)>);

use crate::intro::Cycle;
use crate::intro::TextureAnimate;

pub struct DebugGamePlugin;
impl Plugin for DebugGamePlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_systems(OnEnter(crate::AppState::Game), debug_setup)
                .add_systems(
                    PostUpdate,
                    // (debug_check_mouse, debug_draw).run_if(in_state(crate::AppState::Game)),
                    (debug_draw).run_if(in_state(crate::AppState::Game)),
                );
        }
    }
}

#[derive(Component, Default)]
pub struct DebugUi {
    text: Map<&'static str, String>,
    collisions: Vec<(Collide, Aabb2d)>,
    ctl_aabb: Option<Aabb2d>,
    cursor: Vec2,
}

impl DebugUi {
    fn watch(&mut self, key: &'static str, val: impl std::fmt::Debug) {
        self.text.insert(key, format!("{:?}", val));
    }
}
#[derive(Component)]
pub struct MainCamera;

const TILE_SZ: f32 = 50.;
const MAP: (Vec2, usize, [u8; 27 * 112]) = (
    Vec2::new(-200.0, -400.0),
    27,
    [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 1, // 0
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 1
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 2
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 1, // 3
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 1, // 4
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, // 5
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 1, // 6
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1, // 7
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1, // 8
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, // 9
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 10
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 11
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 12
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 13
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 14
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 15
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 16
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 17
        1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 18
        1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 19
        1, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 20
        1, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 21
        1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 22
        1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 23
        1, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 24
        1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 25
        1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 26
        1, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 27
        1, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 28
        1, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 29
        1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 30
        1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 31
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 32
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 33
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 34
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 35
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 36
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 37
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 38
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 39
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 40
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 41
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 4, 1, 5, 0, 0, 0, 0, 1, // 42
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 43
        1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 44
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 45
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, // 46
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 47
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 48
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1, // 49
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 50
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 51
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, // 52
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 53
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 54
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1, // 55
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 56
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 57
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 58
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 59
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, // 60
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 61
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 62
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 63
        1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 64
        1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 65
        1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 66
        1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 67
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 68
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 69
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 70
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 71
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, // 72
        1, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, // 73
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 74
        1, 0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 75
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, // 76
        1, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 77
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 4, 1, 5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, // 78
        1, 0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, // 79
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, // 80
        1, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, // 81
        1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 82
        1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1, // 83
        1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1, // 84
        1, 0, 0, 0, 0, 0, 1, 1, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1, // 85
        1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 86
        1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 87
        3, 0, 0, 0, 0, 0, 4, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 88
        1, 5, 0, 0, 0, 0, 1, 5, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 89
        3, 0, 0, 0, 0, 4, 1, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 90
        1, 5, 0, 0, 0, 0, 1, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 91
        3, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 92
        1, 5, 0, 0, 0, 0, 1, 5, 0, 1, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 93
        1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 94
        1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 95
        1, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 96
        1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 97
        1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 98
        1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // 99
        1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
        1, // 100
        1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
        1, // 101
        1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0,
        1, // 102
        1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        1, // 103
        1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 3, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        1, // 104
        1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        1, // 105
        1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        1, // 106
        1, 0, 1, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        1, // 107
        1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        1, // 108
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, // 109
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, // 110
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, // 111
    ],
);

impl Tile {
    fn spawn<'c>(
        commands: &'c mut Commands,
        t: u8,
        pos: Vec3,
        tile_types: &TileTypes,
    ) -> bevy::ecs::system::EntityCommands<'c> {
        let &(color, collide, ref tex_cfg) = &tile_types[t as usize];
        let mut rect = None;
        let mut tex = default();
        if let &Some((ref hndl, (w, h), s)) = tex_cfg {
            let u = (pos.x * s / TILE_SZ).rem_euclid(w);
            let v = ((-pos.y) * s / TILE_SZ).rem_euclid(h);
            rect = Some(Rect::new(u, v, u + s, v + s));
            tex = hndl.clone();
        }

        commands.spawn((
            collide,
            Tile(t),
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::ONE),
                    rect,
                    ..default()
                },
                transform: Transform {
                    translation: pos,
                    scale: Vec3::new(TILE_SZ, TILE_SZ, 1.),
                    ..default()
                },
                texture: tex,
                ..default()
            },
        ))
    }
}

fn debug_setup(mut command: Commands) {
    command.spawn((
        DebugUi::default(),
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.),
                left: Val::Px(10.),
                ..default()
            },
            ..default()
        },
    ));
}

pub fn setup(
    mut command: Commands,
    assets: Res<AssetServer>,
    mut win: Query<&mut Window, With<PrimaryWindow>>,
    mut tile_types: ResMut<TileTypes>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    command.spawn((
        MainCamera,
        Camera2dBundle {
            projection: OrthographicProjection {
                near: 1000.,
                far: -1000.,
                scaling_mode: ScalingMode::FixedVertical(600.),
                ..default()
            },
            ..default()
        },
    ));

    if cfg!(debug_assertions) {}

    let layout = TextureAtlasLayout::from_grid(Vec2::new(251., 377.), 3, 2, None, None);
    command.spawn((
        Control,
        Movement::default(),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.2, 1.4)),
                // rect: Some(Rect::new(0., 0., 251., 350.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::new(45., 45., 1.),
                ..default()
            },
            texture: assets.load("baby-idle-sheet.png"),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layouts.add(layout),
            index: 0,
        },
        TextureAnimate {
            frame_len: 0.1,
            cycle: Cycle::PingPong,
            idx_beg: 0,
            idx_end: 4,
        },
    ));

    let garbage_bg = (assets.load("tiled_garbage.png"), (1500., 1000.), 200.);
    command.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.2, 0.2, 0.5),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0., 0., -10.),
            scale: Vec3::splat(0.6),
            ..default()
        },
        texture: garbage_bg.0.clone(),
        ..default()
    });
    tile_types.0.extend([
        (
            Color::rgb(0.5, 0.5, 1.0),
            Collide::Square,
            Some(garbage_bg.clone()),
        ),
        (Color::RED, Collide::StepR, Some(garbage_bg.clone())),
        (Color::BLUE, Collide::StepL, Some(garbage_bg.clone())),
        (Color::ORANGE, Collide::SlopeR, Some(garbage_bg.clone())),
        (Color::GREEN, Collide::SlopeL, Some(garbage_bg.clone())),
        (Color::YELLOW, Collide::Square, None),
    ]);
    let map_origin = MAP.0;
    for (i, &t) in MAP.2.iter().rev().enumerate() {
        if t == 0 {
            continue;
        }
        let (x, y) = (MAP.1 - (i % MAP.1) - 1, i / MAP.1);
        let v = Vec2::new(x as f32, y as f32) * Vec2::splat(TILE_SZ);
        Tile::spawn(&mut command, t, (map_origin + v).extend(0.), &tile_types);
    }

    for mut win in &mut win {
        win.cursor.icon = CursorIcon::Pointer;
        win.cursor.visible = true;
    }
}

pub fn check_kbd(
    kbd: Res<ButtonInput<KeyCode>>,
    mut quit: EventWriter<AppExit>,
    mut ctl: Query<&mut Movement, With<Control>>,
    tiles: Query<(&Transform, &Tile)>,
) {
    if kbd.pressed(KeyCode::Escape) {
        if cfg!(debug_assertions) {
            save_map(tiles);
        }
        quit.send(AppExit);
    }

    let mut vx = 0.;
    let mut vy = 0.;
    if kbd.pressed(KeyCode::ArrowLeft) {
        vx -= 1.;
    }
    if kbd.pressed(KeyCode::ArrowRight) {
        vx += 1.;
    }
    if kbd.pressed(KeyCode::Space) {
        vy += 1.;
    }
    if kbd.pressed(KeyCode::ArrowDown) {
        vy -= 1.;
    }

    let v = Vec2::new(vx, vy);
    for mut c in &mut ctl {
        c.ctl = v * 5.;
    }
}

pub fn debug_check_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut tiles: Query<(
        Entity,
        &Transform,
        &mut Tile,
        &mut Sprite,
        &mut Handle<Image>,
        &mut Collide,
    )>,
    tile_types: Res<TileTypes>,
    mut commands: Commands,
    mut ev_scroll: EventReader<MouseWheel>,
    mut cam_trans: Query<&mut Transform, (With<Camera>, With<MainCamera>, Without<Tile>)>,
    mut dbg: Query<&mut DebugUi>,
) {
    let Some(cursor) = ({
        let (cam, cam_gtrans) = cam.single_mut();
        let Some(cursor) = win.single().cursor_position() else {
            return;
        };
        cam.viewport_to_world_2d(cam_gtrans, cursor)
    }) else {
        return;
    };
    let mut dbg = dbg.single_mut();

    dbg.cursor = cursor;

    if mouse.just_pressed(MouseButton::Left) {
        let cursor_pt = Aabb2d::new(cursor, Vec2::ZERO);
        for (e, trans, mut tile, mut s, mut img, mut col) in &mut tiles {
            let tile_box = Aabb2d::new(trans.translation.xy(), trans.scale.xy() / 2.);
            if !tile_box.contains(&cursor_pt) {
                continue;
            }

            // rotate tile type
            tile.0 = (tile.0 + 1) % (tile_types.len() as u8);
            if tile.0 == 0 {
                // type 0 is special, it means no tile
                commands.get_entity(e).unwrap().despawn();
            } else {
                let (color, collide, tex) = &tile_types.0[tile.0 as usize];
                s.color = *color;
                if let Some((hndl, _, _)) = tex {
                    *img = hndl.clone();
                } else {
                    *img = default();
                }
                *col = *collide;
            }
            return;
        }

        // no tile, need to insert
        let tile_pos = (cursor / TILE_SZ).round() * TILE_SZ;
        Tile::spawn(&mut commands, 1, tile_pos.extend(0.), &tile_types);
    }

    // zoom the camera using the scroll wheel
    // or scale the camera by 1.1 to the power of wheel.y
    let mut zoom = 0.;
    for ev in ev_scroll.read() {
        zoom += ev.y;
    }
    let mut cam_trans = cam_trans.single_mut();
    let zoom = (1.1f32).powf(zoom.round());
    cam_trans.scale *= Vec3::new(zoom, zoom, 1.);
}

// calculate how much we have to push aabb to no longer collide with col
// for instance, if aabb is not intersection col_aabb, then we don't need to push it away at all
// if aabb is intersecting col_aabb, col is square, and it would
//
// col determines the shape and characteristics:
// - Square is a square block. standing on this dampens gravity's pull
// - StepL/R are left or right steps
//   the collider is the shape of left or right triangles,
//   and they allow you to stand on them by dampening gravity
// - SlopeL/R are left or right slopes,
//   the collider is the shape of left or right triangles,
//   but standing on them does not dampen gravity
//
fn collide_push(aabb: &Aabb2d, col: &Collide, col_aabb: &Aabb2d) -> (Vec2, bool, bool) {
    let lt = col_aabb.min.x - aabb.max.x;
    let rt = col_aabb.max.x - aabb.min.x;
    let up = col_aabb.max.y - aabb.min.y;
    let dn = col_aabb.min.y - aabb.max.y;
    let horz = if lt.abs() < rt.abs() { lt } else { rt };
    let vert = if dn.abs() < up.abs() { dn } else { up };

    use std::f32::consts::FRAC_1_SQRT_2;
    // normalized vectors
    const UNIT_DN_RT: Vec2 = Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2); // y = -x
    const UNIT_DN_LT: Vec2 = Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2); // y = x
    let pt_line_dist = |left, p: Vec2| {
        let a = col_aabb.center();
        let n = if left { UNIT_DN_RT } else { UNIT_DN_LT };
        // wikipedia taught me how to do this
        let dist = (p - a - ((p - a).dot(n) * n)).length();
        // does point lie above or below the line
        let sign = if left {
            (p - a).y > -(p - a).x
        } else {
            (p - a).y > (p - a).x
        };
        (dist, sign)
    };

    match col {
        Collide::Square => {
            if horz.abs() > vert.abs() {
                (Vec2::new(0., vert), false, true)
            } else {
                (Vec2::new(horz, 0.), true, false)
            }
        }
        Collide::StepL | Collide::SlopeL => {
            // collide like a left triangle |\
            let (dist, sign) = pt_line_dist(true, aabb.min);
            if sign {
                return (Vec2::ZERO, false, false);
            }
            let dampv = matches!(col, Collide::StepL) || vert <= 0.;
            let (vert_dist, vert_v) = (vert.abs(), (Vec2::new(0., vert), false, dampv));
            let (horz_dist, horz_v) = (horz.abs(), (Vec2::new(horz, 0.), true, false));
            let diag_v = (
                Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2) * dist,
                false,
                matches!(col, Collide::StepL),
            );
            if dampv && vert_dist < horz_dist && vert_dist < dist {
                vert_v
            } else if horz_dist < dist {
                horz_v
            } else {
                diag_v
            }
        }
        Collide::StepR | Collide::SlopeR => {
            // collide like a right triangle /|
            let p = Vec2::new(aabb.max.x, aabb.min.y);
            let (dist, sign) = pt_line_dist(false, p);
            if sign {
                return (Vec2::ZERO, false, false);
            }
            let dampv = matches!(col, Collide::StepR) || vert <= 0.;
            let (vert_dist, vert_v) = (vert.abs(), (Vec2::new(0., vert), false, dampv));
            let (horz_dist, horz_v) = (horz.abs(), (Vec2::new(horz, 0.), true, false));
            let diag_v = (
                Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2) * dist,
                false,
                matches!(col, Collide::StepR),
            );
            if dampv && vert_dist < horz_dist && vert_dist < dist {
                vert_v
            } else if horz_dist < dist {
                horz_v
            } else {
                diag_v
            }
        }
    }
}

// the intent is to cast the ctl's aabb along ctl's velocity and check for any collisions
// if there are any collisions, then reduce velocity until there aren't
//
// this is not working correctly as it sees collisions where it shouldn't
pub fn check_collide(
    time: Res<Time>,
    mut update_rem: ResMut<PhysicsTick>,
    mut ctl: Query<(&Transform, &mut Movement), With<Control>>,
    col: Query<(&Transform, &Collide)>,
    mut dbg: Query<&mut DebugUi>,
) {
    let (t, mut v) = ctl.single_mut();
    if v.ctl + v.force == Vec2::ZERO {
        v.out = Vec2::ZERO;
        return;
    }

    let mut dt = update_rem.0;
    // 60 physics ticks a second
    dt += time.delta_seconds() * 60.;
    let mut aabb = Aabb2d::new(t.translation.xy(), t.scale.xy() / 2.);
    v.climb = false;
    let mut collisions = vec![];
    let mut pushes = vec![];
    while dt > 1. {
        collisions = vec![];
        v.force += Vec2::new(0., -9.8 / 60.);
        aabb = Aabb2d::new(aabb.center() + v.ctl.xy() + v.force.xy(), aabb.half_size());
        for (col, &c) in &col {
            let col_aabb = Aabb2d::new(col.translation.xy(), col.scale.xy() / 2.);
            if aabb.intersects(&col_aabb) {
                collisions.push((
                    (col_aabb.center() - aabb.center()).length_squared(),
                    c,
                    col_aabb,
                ));
            }
        }

        // sort by distance to aabb
        collisions.sort_by(|c1, c2| c1.0.total_cmp(&c2.0));

        // three tries outta be enough
        for i in 0..3 {
            let mut pushed = false;
            for (_, col, col_aabb) in &collisions {
                if !aabb.intersects(col_aabb) {
                    continue;
                }
                let (push, damph, dampv) = collide_push(&aabb, col, col_aabb);
                if push == Vec2::ZERO {
                    continue;
                }
                pushes.push((i, *col, push, damph, dampv));

                if dampv {
                    if v.ctl.y > 0. && push.y < 0. {
                        v.climb = true;
                    }
                    v.force.y = 0.;
                }
                if damph {
                    if push.x.signum() != v.force.x.signum() {
                        v.force.x = 0.;
                    }
                }
                aabb.min += push;
                aabb.max += push;
                pushed = true;
            }
            if !pushed {
                break;
            }
        }
        dt -= 1.;
    }
    if cfg!(debug_assertions) && !collisions.is_empty() {
        let mut dbg = dbg.single_mut();
        dbg.watch("vctl", v.ctl);
        dbg.watch("vforce", v.force);
        dbg.watch("pos", t.translation);
        dbg.watch("rot", t.rotation.to_axis_angle());
        dbg.watch("climb", v.climb);
        dbg.watch("pushes", pushes);
        dbg.collisions = collisions
            .into_iter()
            .map(|(_, c, aabb)| (c, aabb))
            .collect();
        dbg.ctl_aabb = Some(aabb);
    }

    let tnew = aabb.center();
    v.out = tnew - t.translation.xy();
    if dt != update_rem.0 {
        update_rem.0 = dt;
    }
}

pub fn update_movement(mut movers: Query<(&mut Transform, &Movement, &mut Sprite)>) {
    for (mut t, v, mut s) in &mut movers {
        t.translation.x += v.out.x;
        t.translation.y += v.out.y;

        if !v.climb {
            t.rotation = Quat::IDENTITY;
            if v.ctl.x < 0. {
                s.flip_x = true;
            } else if v.ctl.x > 0. {
                s.flip_x = false;
            }
        } else {
            s.flip_x = true;
            t.rotation = Quat::from_rotation_z(-PI / 2.);
        }

        // kill box
        if t.translation.y < -1000. {
            t.translation = Vec3::ZERO;
        }
    }
}

pub fn pan_camera(
    mut cam: Query<&mut Transform, (With<Camera>, Without<Control>)>,
    ctl: Query<&Transform, With<Control>>,
) {
    // move the camera to track the player when he gets too close to the edge of the window
    let ctl = ctl.single().translation;
    let mut cam = cam.single_mut();
    // hardcoded 100x100 pixel box
    let cam_bound = 100.;
    if (ctl.x - cam.translation.x).abs() > cam_bound {
        let dx = ctl.x - cam.translation.x;
        if dx < 0. {
            cam.translation.x += dx + cam_bound;
        } else {
            cam.translation.x += dx - cam_bound;
        }
    }
    if (ctl.y - cam.translation.y).abs() > cam_bound {
        let dy = ctl.y - cam.translation.y;
        if dy < 0. {
            cam.translation.y += dy + cam_bound;
        } else {
            cam.translation.y += dy - cam_bound;
        }
    }
}

pub fn animate_texture(mut tex: Query<(&mut TextureAtlas, &TextureAnimate)>, time: Res<Time>) {
    for (mut atlas, anim) in &mut tex {
        let (beg, end) = (anim.idx_beg, anim.idx_end);
        let len = end + 1 - beg;
        let n = time.elapsed_seconds() / anim.frame_len;
        let n = n as usize;
        match anim.cycle {
            Cycle::PingPong => {
                let n = n % (len * 2 - 2);
                if n < len {
                    atlas.index = beg + n;
                } else {
                    atlas.index = beg + (len - (n - len) - 2);
                }
            }
            Cycle::Loop => {
                let n = n % len;
                atlas.index = beg + n;
            }
        }
    }
}

pub fn debug_draw(mut gizmos: Gizmos, mut dbg: Query<(&mut Text, &DebugUi)>) {
    let (mut txt, dbg) = dbg.single_mut();
    txt.sections = (dbg.text.iter())
        .map(|(k, v)| TextSection::new(format!("{k}: {v}\n"), default()))
        .collect();
    if !dbg.collisions.is_empty() {
        let n = {
            let n = dbg.collisions.len() - 1;
            if n == 0 {
                1.
            } else {
                0.5 / n as f32
            }
        };
        for (i, (col, aabb)) in dbg.collisions.iter().enumerate() {
            let color = Color::rgb(1. - (i as f32 * n), 0., 0.);
            match col {
                Collide::Square => {
                    gizmos.rect_2d(aabb.center(), 0., aabb.max - aabb.min, color);
                }
                Collide::StepL | Collide::SlopeL => {
                    gizmos.linestrip_2d(
                        [
                            aabb.min,
                            Vec2::new(aabb.max.x, aabb.min.y),
                            Vec2::new(aabb.min.x, aabb.max.y),
                            aabb.min,
                        ],
                        color,
                    );
                }
                Collide::StepR | Collide::SlopeR => {
                    gizmos.linestrip_2d(
                        [
                            aabb.min,
                            Vec2::new(aabb.max.x, aabb.min.y),
                            aabb.max,
                            aabb.min,
                        ],
                        color,
                    );
                }
            }
        }
        if let Some(aabb) = &dbg.ctl_aabb {
            gizmos.rect_2d(aabb.center(), 0., aabb.half_size() * 2., Color::GREEN);
        }
    }
    let cursor = (dbg.cursor / TILE_SZ).round() * TILE_SZ;
    gizmos.rect_2d(cursor, 0., Vec2::new(TILE_SZ, TILE_SZ), Color::GREEN);
}

pub fn save_map(tiles: Query<(&Transform, &Tile)>) {
    let mut data: Vec<_> = tiles
        .iter()
        .map(|(t, s)| (t.translation.xy(), *s))
        .collect();
    data.sort_by(|(t1, _), (t2, _)| match t1.y.total_cmp(&t2.y) {
        std::cmp::Ordering::Equal => t1.x.total_cmp(&t2.x),
        c => c,
    });
    let mut min = data[0].0;
    let mut max = data[data.len() - 1].0;
    for (d, _) in &data {
        if d.x < min.x {
            min.x = d.x;
        }
        if d.x > max.x {
            max.x = d.x;
        }
    }

    let width = ((max.x - min.x) / TILE_SZ) as usize + 1;
    let height = ((max.y - min.y) / TILE_SZ) as usize + 1;
    println!("const MAP: (Vec2, usize, [u8; {width} * {height}]) = (");
    println!("  Vec2::new({:?}, {:?}),", min.x.floor(), min.y.floor());
    println!("  {width},");
    println!("  [");
    let mut map = vec![vec![0u8; width]; height];
    for (trans, tile) in data {
        let trans = (trans - min) / TILE_SZ;
        map[trans.y as usize][trans.x as usize] = tile.0;
    }
    for (y, row) in map.iter().rev().enumerate() {
        print!("    ");
        for t in row {
            print!("{t}, ");
        }
        println!(" // {y}");
    }
    println!("  ],");
    println!(");");

    const BMP_SZ: usize = 0x02;
    const BMP_PX_W: usize = 0x12;
    const BMP_PX_H: usize = 0x16;
    const BMP_DATA_SZ: usize = 0x22;
    const BMP_START_DATA: usize = 0x36;
    let mut bmp_buf = vec![
        // BMP Header
        0x42, 0x4D, // "BM"
        0x00, 0x00, 0x00, 0x00, // size (todo)
        0x00, 0x00, // (unused)
        0x00, 0x00, // (unused)
        0x36, 0x00, 0x00, 0x00, // offset to pixel array
        // DIB Header
        0x28, 0x00, 0x00, 0x00, // size of DIB header
        0x00, 0x00, 0x00, 0x00, // width of bitmap in pixels (todo)
        0x00, 0x00, 0x00, 0x00, // height of bitmap in pixels (todo)
        0x01, 0x00, // # of color planes
        0x18, 0x00, // # of bits per-pixel (24 bit)
        0x00, 0x00, 0x00, 0x00, // compression (unused)
        0x00, 0x00, 0x00, 0x00, // size of bitmap data (todo)
        0x13, 0x0B, 0x00, 0x00, // print resolution (default)
        0x13, 0x0B, 0x00, 0x00, // print resolution (default)
        0x00, 0x00, 0x00, 0x00, // # of colors in palette
        0x00, 0x00, 0x00, 0x00, // (unused)
              // pixel array/bitmap data
    ];
    for row in map.iter() {
        for x in row {
            match x {
                0 => bmp_buf.extend([0x00, 0x00, 0x00]), // black
                1 => bmp_buf.extend([0xff, 0xff, 0xff]), // white
                2 => bmp_buf.extend([0x00, 0x00, 0xff]), // red
                3 => bmp_buf.extend([0xff, 0x00, 0x00]), // blue
                4 => bmp_buf.extend([0x00, 0xff, 0x00]), // green
                5 => bmp_buf.extend([0x00, 0x88, 0xff]), // orange
                6 => bmp_buf.extend([0x00, 0xff, 0xff]), // yellow
                _ => unimplemented!(),
            }
        }
        let pad = (row.len() * 3) % 4;
        if pad != 0 {
            let pad = 4 - pad;
            for _ in 0..pad {
                bmp_buf.push(0x00);
            }
        }
    }
    let data_sz = bmp_buf.len() - BMP_START_DATA;
    let file_sz = bmp_buf.len();
    let px_w = if (map[0].len() % 4) != 0 {
        map[0].len() + 4 - (map[0].len() % 4)
    } else {
        map[0].len()
    };
    let px_h = map.len();

    use std::io::Write as _;
    for (off, val) in [
        (BMP_SZ, file_sz),
        (BMP_PX_W, px_w),
        (BMP_PX_H, px_h),
        (BMP_DATA_SZ, data_sz),
    ] {
        (&mut bmp_buf[off..])
            .write(&(val as u32).to_le_bytes())
            .unwrap();
    }
    std::fs::write("./map.bmp", bmp_buf).unwrap();
}
