use bevy::{
    audio::PlaybackMode,
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::collections::HashMap as Map;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;
// surely this should be wide enough
const LETTERBOX_WIDTH: f32 = 2000.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Baby".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(CueSequencer {
            playing: true,
            ..default()
        })
        .add_systems(Startup, (setup, setup_anim))
        .add_systems(Update, (sequence_cues, animate_texture))
        // .add_systems(Update, (sprite_animation, sound_player, volume, draw_debug))
        .run();
}

// Clamped inverse LERP
// https://www.gamedev.net/articles/programming/general-and-gameplay-programming/inverse-lerp-a-super-useful-yet-often-overlooked-function-r5230/
fn inv_lerp(a: f32, b: f32, x: f32) -> f32 {
    if x < a {
        0.
    } else if x > b {
        1.
    } else {
        (x - a) / (b - a)
    }
}

#[derive(Component)]
struct DebugText;

fn draw_debug(mut text: Query<&mut Text, With<DebugText>>, time: Res<Time>) {
    for mut t in &mut text {
        *t = Text::from_section(
            format!("time: {:.3}", time.elapsed_seconds()),
            TextStyle::default(),
        );
    }
}

// Schedule entity despawn
#[derive(Component)]
struct DespawnTimer(Timer);

fn despawn(mut commands: Commands, mut query: Query<(Entity, &mut DespawnTimer)>, time: Res<Time>) {
    for (entity, mut despawn_timer) in &mut query {
        despawn_timer.0.tick(time.delta());
        if despawn_timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ------------------------------- Sprite Animation -------------------------------
// Indices representing a sprite sheet
#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
enum SpriteAnimationType {
    // play to end, repeat
    // 123123123
    Linear,

    // play to end, go backwards, repeat
    // 123212321
    PingPong(PingPongState),
}

impl SpriteAnimationType {
    fn new_ping_pong() -> Self {
        Self::PingPong(PingPongState::default())
    }
}

enum PingPongState {
    Forward,
    Backward,
}

impl PingPongState {
    fn default() -> Self {
        PingPongState::Forward
    }
}

#[derive(Component, Deref, DerefMut)]
struct SpriteAnimationTimer(Timer);

fn sprite_animation(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut SpriteAnimationType,
        &mut SpriteAnimationTimer,
        &mut TextureAtlas,
    )>,
) {
    for (indices, mut anim_type, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            match *anim_type {
                SpriteAnimationType::Linear => {
                    atlas.index = if atlas.index == indices.last {
                        indices.first
                    } else {
                        atlas.index + 1
                    }
                }
                SpriteAnimationType::PingPong(ref mut ppstate) => match ppstate {
                    PingPongState::Forward => {
                        atlas.index = if atlas.index == indices.last {
                            *ppstate = PingPongState::Backward;
                            atlas.index - 1
                        } else {
                            atlas.index + 1
                        }
                    }
                    PingPongState::Backward => {
                        atlas.index = if atlas.index == indices.first {
                            *ppstate = PingPongState::Forward;
                            atlas.index + 1
                        } else {
                            atlas.index - 1
                        }
                    }
                },
            }
        }
    }
}

// ------------------------------- Sound -------------------------------
#[derive(Component)]
enum SoundVolume {
    Background(f32),
    CarIdle(f32),
}

#[derive(Component)]
struct SoundPlayTimer(Timer);

fn volume(query: Query<(&AudioSink, &SoundVolume)>, time: Res<Time>) {
    for (sink, sound) in &query {
        match sound {
            SoundVolume::Background(base_volume) => sink.set_volume(
                base_volume * (time.elapsed_seconds() / KEYFRAME_BG_MUSIC_VOL_MAX).min(1.0),
            ),
            SoundVolume::CarIdle(base_volume) => sink.set_volume(
                base_volume
                    * inv_lerp(
                        KEYFRAME_CAR_SND_IDLE_START,
                        KEYFRAME_CAR_SND_IDLE_VOL_MAX,
                        time.elapsed_seconds(),
                    ),
            ),
        }
    }
}

fn sound_player(mut query: Query<(&AudioSink, &mut SoundPlayTimer)>, time: Res<Time>) {
    for (sink, mut sound_play_timer) in &mut query {
        sound_play_timer.0.tick(time.delta());
        if sound_play_timer.0.just_finished() {
            sink.play();
        }
    }
}

// ------------------------------- Intro Cutscene -------------------------------
enum Q {
    // advance time
    Tick(f32),
    // set translation
    Tran(&'static str, f32, f32),
    // sound paused
    Paused(&'static str, bool),
    // sound volume
    Vol(&'static str, f32),
    // despawn
    Despawn(&'static str),
}
enum AR {
    Sprite(
        &'static str,
        &'static str,
        (f32, f32, usize, usize, f32, Cycle, usize, usize),
        f32,
        bool,
    ),
    Sound(&'static str, &'static str, bool),
    Overlay(&'static str, f32),
    Image(&'static str, &'static str, (f32, f32, f32), f32),
}
const ANIM_RSC: &'static [AR] = &[
    AR::Image("bg", "scenes/intro/bg.png", (0., 0., -10.), 1.),
    AR::Image("pile1", "scenes/intro/pile_1.png", (0., 0., 10.), 1.),
    AR::Image("pile2", "scenes/intro/pile_2.png", (0., 0., 10.), 1.),
    AR::Sprite(
        "car",
        "car-sheet.png",
        (170., 100., 3, 4, 0.11, Cycle::Loop, 1, 6),
        1.5,
        true,
    ),
    AR::Sprite(
        "baby",
        "baby-idle-sheet.png",
        (251., 377., 3, 2, 0.1, Cycle::PingPong, 0, 4),
        0.5,
        false,
    ),
    AR::Overlay("screen", 100.),
    AR::Sound("city", "sounds/city-background.wav", false),
    AR::Sound("car_idle", "sounds/car-idle.wav", false),
    AR::Sound("car_brake", "sounds/car-brake-squeak.wav", true),
    AR::Sound("car_win", "sounds/car-window-open.wav", true),
];
const ANIM_CUE: &'static [Q] = &[
    Q::Tran("baby", 0., -200.),
    Q::Vol("city", 0.),
    Q::Paused("city", false),
    Q::Paused("car_idle", true),
    Q::Vol("car_idle", 0.),
    Q::Paused("car_brake", true),
    Q::Paused("car_win", true),
    //
    Q::Tick(3.),
    Q::Vol("city", 1.),
    Q::Tick(1.),
    Q::Despawn("screen"),
    //
    Q::Tick(2.),
    Q::Tran("car", 700., -50.),
    Q::Paused("car_idle", false),
    Q::Tick(3.),
    Q::Vol("car_idle", 0.2),
    //
    Q::Tick(1.75),
    Q::Paused("car_brake", false),
    Q::Tick(0.25),
    Q::Tran("car", -50., -150.),
    Q::Tick(1.),
    Q::Paused("car_win", false),
    //
    Q::Tick(5.),
    // baby thrown
];

#[derive(Component, Clone, Copy)]
struct TextureAnimate {
    frame_len: f32,
    cycle: Cycle,
    idx_beg: usize,
    idx_end: usize,
}
#[derive(Clone, Copy)]
enum Cycle {
    PingPong,
    Loop,
}
#[derive(Resource, Default)]
struct CueSequencer {
    audio: Map<Name, (Vec<(f32, f32)>, Vec<(f32, bool)>)>,
    despawn: Map<Name, f32>,
    time: f32,
    playing: bool,
}

impl CueSequencer {
    fn get_curve<T: Copy>(curve: &Vec<(f32, T)>, time: f32) -> Option<(T, T, f32)> {
        if curve.is_empty() {
            return None;
        }
        let mut b = curve[curve.len() - 1];
        let mut a = b;
        for i in 0..curve.len() {
            if time < curve[i].0 {
                b = curve[i];
                if i == 0 {
                    a = b;
                } else {
                    a = curve[i - 1];
                }
                let s = (time - a.0) / (b.0 - a.0);
                return Some((a.1, b.1, s));
            }
        }
        return Some((a.1, b.1, 1.));
    }

    fn get_audio(&mut self, name: &Name, time: f32) -> Option<(f32, bool)> {
        let Some((vol, paused)) = self.audio.get(name) else {
            return None;
        };
        let (vol_a, vol_b, s) = Self::get_curve(vol, time).unwrap_or((1., 1., 1.));
        let vol = vol_b * s + vol_a * (1. - s);
        let (paused, paused_b, s) = Self::get_curve(paused, time).unwrap_or((true, true, 1.));
        let paused = if s >= 1. { paused_b } else { paused };
        Some((vol, paused))
    }

    fn get_despawn(&mut self, name: &Name, time: f32) -> bool {
        if let Some(t) = self.despawn.get(name) {
            return time >= *t;
        }
        return false;
    }
}

fn sequence_cues(
    mut names: Query<(Entity, &Name)>,
    audio: Query<&AudioSink>,
    mut commands: Commands,
    mut sequence: ResMut<CueSequencer>,
    time: Res<Time>,
) {
    if !sequence.playing {
        return;
    }

    sequence.time += time.delta_seconds();
    let t = sequence.time;
    for (e, name) in &mut names {
        if let Some((vol, paused)) = sequence.get_audio(name, t) {
            if let Ok(sink) = audio.get(e) {
                sink.set_volume(vol);
                if sink.is_paused() && !paused {
                    sink.play();
                }
            }
        }
        if sequence.get_despawn(name, t) {
            if let Some(mut ecmd) = commands.get_entity(e) {
                ecmd.despawn();
            }
        }
    }
}

fn animate_texture(mut tex: Query<(&mut TextureAtlas, &TextureAnimate)>, time: Res<Time>) {
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

fn setup_anim(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut sequence: ResMut<CueSequencer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut pos: Map<&'static str, Vec3> = Map::new();
    for cue in ANIM_CUE.iter() {
        match cue {
            Q::Tran(name, x, y) => {
                if !pos.contains_key(name) {
                    pos.insert(name, Vec3::new(*x, *y, 0.));
                }
            }
            _ => {}
        }
    }

    let mut entities: Map<Name, Entity> = Map::new();
    for ar in ANIM_RSC.iter() {
        match ar {
            &AR::Sprite(
                name,
                tex,
                (width, height, cols, rows, frame_len, cycle, idx_beg, idx_end),
                scale,
                flip_x,
            ) => {
                let layout =
                    TextureAtlasLayout::from_grid(Vec2::new(width, height), cols, rows, None, None);
                let name = Name::new(name);
                let layout = texture_atlas_layouts.add(layout);
                let trans = pos.get(name.as_str()).cloned().unwrap_or_default();
                let cmd = commands.spawn((
                    name.clone(),
                    SpriteBundle {
                        sprite: Sprite {
                            flip_x,
                            ..default()
                        },
                        transform: Transform {
                            translation: trans,
                            scale: Vec3::new(scale, scale, 1.),
                            ..default()
                        },
                        texture: asset_server.load(tex),
                        ..default()
                    },
                    TextureAtlas { layout, index: 0 },
                    TextureAnimate {
                        frame_len,
                        cycle,
                        idx_beg,
                        idx_end,
                    },
                ));
                entities.insert(name, cmd.id());
            }
            &AR::Image(name, tex, (x, y, z), s) => {
                let cmd = commands.spawn((
                    Name::new(name),
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(x, y, z),
                            scale: Vec3::new(s, s, 1.),
                            ..default()
                        },
                        texture: asset_server.load(tex),
                        ..default()
                    },
                ));
                entities.insert(Name::new(name), cmd.id());
            }
            &AR::Overlay(name, z) => {
                let cmd = commands.spawn((
                    Name::new(name),
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Rectangle::new(WINDOW_WIDTH, WINDOW_HEIGHT))),
                        material: materials.add(Color::BLACK),
                        transform: Transform::from_xyz(0., 0., z),
                        ..default()
                    },
                ));
                entities.insert(Name::new(name), cmd.id());
            }
            &AR::Sound(name, snd, once) => {
                let cmd = commands.spawn((
                    Name::new(name),
                    AudioBundle {
                        source: asset_server.load(snd),
                        settings: PlaybackSettings {
                            paused: true,
                            mode: if once {
                                PlaybackMode::Once
                            } else {
                                PlaybackMode::Loop
                            },
                            ..default()
                        },
                    },
                ));
                entities.insert(Name::new(name), cmd.id());
            }
        }
    }

    for (name, eid) in &entities {
        let mut t = 0.;

        let mut pos_next = None;
        let mut pos_steps = vec![];
        let mut pos_frames = vec![];

        let mut paused_next = None;
        let mut vol_next = None;
        let mut vol_cues = vec![];
        let mut play_cues = vec![];

        let mut despawn = None;

        for cue in ANIM_CUE.iter() {
            match cue {
                Q::Tran(kname, x, y) if *kname == name.as_str() => {
                    pos_next = Some(Vec3::new(*x, *y, 0.));
                }
                Q::Paused(kname, paused) if *kname == name.as_str() => {
                    paused_next = Some(*paused);
                }
                Q::Vol(kname, vol) if *kname == name.as_str() => {
                    vol_next = Some(*vol);
                }
                Q::Despawn(kname) if *kname == name.as_str() => {
                    despawn = Some(t);
                }
                Q::Tick(dt) => {
                    if let Some(pos_next) = pos_next.take() {
                        pos_frames.push(pos_next);
                        pos_steps.push(t);
                    }
                    if let Some(vol) = vol_next.take() {
                        vol_cues.push((t, vol));
                    }
                    if let Some(paused) = paused_next.take() {
                        play_cues.push((t, paused));
                    }
                    t += dt;
                }
                _ => {}
            }
        }

        if let Some(pos_next) = pos_next {
            pos_frames.push(pos_next);
            pos_steps.push(t);
        }

        if let Some(vol) = vol_next.take() {
            vol_cues.push((t, vol));
        }
        if let Some(paused) = paused_next.take() {
            play_cues.push((t, paused));
        }

        if let Some(t) = despawn {
            sequence.despawn.insert(name.clone(), t);
        }

        if !pos_frames.is_empty() {
            let mut anim = AnimationClip::default();
            anim.add_curve_to_path(
                EntityPath {
                    parts: vec![name.clone()],
                },
                VariableCurve {
                    keyframe_timestamps: pos_steps,
                    keyframes: Keyframes::Translation(pos_frames),
                    interpolation: Interpolation::Linear,
                },
            );

            let mut player = AnimationPlayer::default();
            player.play(animations.add(anim));
            commands.entity(*eid).insert(player);
        }

        if !(vol_cues.is_empty() && play_cues.is_empty()) {
            sequence.audio.insert(name.clone(), (vol_cues, play_cues));
        }
    }
}

const BG_MUSIC_VOL_BASE: f32 = 0.8;
const CAR_IDLE_VOL_BASE: f32 = 0.2;

// background sound is already playing when scene starts, but it's muted so we
// fade into it slightly
const KEYFRAME_BG_MUSIC_VOL_MAX: f32 = 5.0;

// scene reveal
const KEYFRAME_SCENE_REVEAL: f32 = 4.0;

// car moves into frame, idle volume gets louder
const KEYFRAME_CAR_MOVE_START: f32 = 5.0;
const KEYFRAME_CAR_SND_IDLE_START: f32 = 5.0;

// car stops
const KEYFRAME_CAR_MOVE_STOP: f32 = 10.0;
const KEYFRAME_CAR_SND_IDLE_VOL_MAX: f32 = 8.0;

// brake squeak
const KEYFRAME_CAR_SND_BRAKE: f32 = 9.75;

// car window roll down
const KEYFRAME_CAR_SND_WINDOW: f32 = 11.0;

// baby thrown from car
const KEYFRAME_BABY_THROWN: f32 = 14.5;

// baby hits ground, thump
const KEYFRAME_BABY_GROUND: f32 = 15.5;

// car window roll up

// car turns around
// car burnout
// car sound fades

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            // When creating our own OrthographicProjection we need to set the far and near
            // values ourselves.
            // See: https://bevy-cheatbook.github.io/2d/camera.html#caveat-nearfar-values
            far: 1000.,
            near: -1000.,
            scaling_mode: ScalingMode::FixedVertical(600.),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        DebugText,
        Text2dBundle {
            text: Text::from_section("hello, baby!", TextStyle::default()),
            text_anchor: bevy::sprite::Anchor::TopLeft,
            transform: Transform {
                translation: Vec3::new(-380., 280., 101.),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));

    // Scene
    // commands.spawn(SpriteBundle {
    //     texture: asset_server.load("scenes/intro/bg.png"),
    //     transform: Transform::from_xyz(0., 0., 0.),
    //     ..default()
    // });
    // commands.spawn(SpriteBundle {
    //     texture: asset_server.load("scenes/intro/pile_1.png"),
    //     transform: Transform::from_xyz(0., 0., 10.),
    //     ..default()
    // });
    // commands.spawn(SpriteBundle {
    //     texture: asset_server.load("scenes/intro/pile_2.png"),
    //     transform: Transform::from_xyz(0., 0., 10.),
    //     ..default()
    // });

    // Vertical Letterboxes
    let letterbox_h_offset = (WINDOW_WIDTH + LETTERBOX_WIDTH) / 2.;
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(LETTERBOX_WIDTH, WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(letterbox_h_offset, 0., 100.),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(LETTERBOX_WIDTH, WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(-letterbox_h_offset, 0., 100.),
        ..default()
    });

    // Black Screen
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: Mesh2dHandle(meshes.add(Rectangle::new(WINDOW_WIDTH, WINDOW_HEIGHT))),
    //         material: materials.add(Color::BLACK),
    //         transform: Transform::from_xyz(0., 0., 100.),
    //         ..default()
    //     },
    //     DespawnTimer(Timer::from_seconds(KEYFRAME_SCENE_REVEAL, TimerMode::Once)),
    // ));

    // spawn_car(
    //     &mut commands,
    //     &asset_server,
    //     &mut texture_atlas_layouts,
    //     &mut animations,
    // );
    // spawn_baby(&mut commands, &asset_server, &mut texture_atlas_layouts);

    /*
    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/city-background.wav"),
            settings: PlaybackSettings {
                paused: false,
                volume: Volume::ZERO,
                ..default()
            },
        },
        SoundVolume::Background(BG_MUSIC_VOL_BASE),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/car-idle.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Loop,
                ..default()
            },
        },
        SoundVolume::CarIdle(CAR_IDLE_VOL_BASE), // control base volume
        SoundPlayTimer(Timer::from_seconds(
            KEYFRAME_CAR_SND_IDLE_START,
            TimerMode::Once,
        )),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/car-brake-squeak.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            },
        },
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_CAR_SND_BRAKE, TimerMode::Once)),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/car-window-open.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            },
        },
        SoundPlayTimer(Timer::from_seconds(
            KEYFRAME_CAR_SND_WINDOW,
            TimerMode::Once,
        )),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/baby-throw-woosh.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            },
        },
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_BABY_THROWN, TimerMode::Once)),
    ));
    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/thump.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            },
        },
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_BABY_GROUND, TimerMode::Once)),
    ));
    */
}

fn spawn_car(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    animations: &mut ResMut<Assets<AnimationClip>>,
) {
    let layout = TextureAtlasLayout::from_grid(Vec2::new(170., 100.), 3, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let sprite_animation_indices = AnimationIndices { first: 1, last: 6 };

    let car_name = Name::new("car");
    let mut car_animation = AnimationClip::default();
    car_animation.add_curve_to_path(
        EntityPath {
            parts: vec![car_name.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![KEYFRAME_CAR_MOVE_START, KEYFRAME_CAR_MOVE_STOP],
            keyframes: Keyframes::Translation(vec![
                Vec3::new(700., -50., 1.),
                Vec3::new(-50., -150., 1.),
            ]),
            interpolation: Interpolation::Linear,
        },
    );
    let mut player = AnimationPlayer::default();
    player.play(animations.add(car_animation));

    commands.spawn((
        car_name,
        SpriteBundle {
            texture: asset_server.load("car-sheet.png"),
            transform: Transform::from_xyz(700., -50., 1.).with_scale(Vec3::ONE * 1.5),
            sprite: Sprite {
                flip_x: true,
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: sprite_animation_indices.first,
        },
        sprite_animation_indices,
        SpriteAnimationType::Linear,
        player,
        SpriteAnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn spawn_baby(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(Vec2::new(251., 377.), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let sprite_animation_indices = AnimationIndices { first: 0, last: 4 };

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("baby-idle-sheet.png"),
            transform: Transform::from_xyz(-0., -200., 2.).with_scale(Vec3::ONE * 0.5),
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: sprite_animation_indices.first,
        },
        sprite_animation_indices,
        SpriteAnimationType::new_ping_pong(),
        SpriteAnimationTimer(Timer::from_seconds(0.11, TimerMode::Repeating)),
    ));
}
