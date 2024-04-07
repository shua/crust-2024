use bevy::{
    audio::PlaybackMode,
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_editor_pls::egui::{emath::inverse_lerp, lerp};
use std::collections::HashMap as Map;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;
// surely this should be wide enough
const PILLARBOX_WIDTH: f32 = 2000.;

#[derive(Component, Default)]
struct DebugUi {
    text: Map<&'static str, String>,
}

impl DebugUi {
    fn watch(&mut self, key: &'static str, val: impl std::fmt::Debug) {
        self.text.insert(key, format!("{:?}", val));
    }
}

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
        .add_systems(
            Update,
            (sequence_cues, sequence_camera, animate_texture, draw_debug),
        )
        .add_systems(PostUpdate, draw_debug)
        .run();
}

fn draw_debug(mut dbg: Query<(&mut Text, &DebugUi)>) {
    let (mut txt, dbg) = dbg.single_mut();
    txt.sections = (dbg.text.iter())
        .map(|(k, v)| TextSection::new(format!("{k}: {v}\n"), default()))
        .collect();
}

// ------------------------------- Intro Cutscene -------------------------------
enum Q {
    // advance time
    Tick(f32),
    // set translation
    Tran(&'static str, f32, f32),
    // set rotation (in radians around z-axis)
    Rot(&'static str, f32),
    // set flip x
    Flip(&'static str, bool),
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
    AR::Image("bg", "scenes/intro/bg.png", (0., -35., -10.), 1.),
    AR::Image("pile1", "scenes/intro/pile_1.png", (0., -35., 10.), 1.),
    AR::Image("pile2", "scenes/intro/pile_2.png", (0., -35., 10.), 1.),
    AR::Image("pile2", "scenes/intro/pile_2.png", (0., -35., 10.), 1.),
    AR::Image("baby_thrown", "baby-thrown.png", (0., 0., -10.), 0.4),
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
    AR::Sound("car_win_open", "sounds/car-window-open.wav", true),
    AR::Sound("car_win_close", "sounds/car-window-close.wav", true),
    AR::Sound("woosh", "sounds/woosh.wav", true),
    AR::Sound("thump", "sounds/thump.wav", true),
    AR::Sound("car_peels_out", "sounds/car-peels-out.wav", true),
];
const ANIM_CUE: &'static [Q] = &[
    Q::Tran("baby", 60., -200.),
    Q::Vol("city", 0.),
    Q::Paused("city", false),
    Q::Paused("car_idle", true),
    Q::Paused("car_brake", true),
    Q::Paused("car_win_open", true),
    Q::Paused("car_win_close", true),
    Q::Paused("car_peels_out", true),
    Q::Paused("woosh", true),
    Q::Paused("thump", true),
    // background soundscape fades in
    Q::Tick(3.),
    Q::Vol("city", 0.8),
    // scene reveal
    Q::Tick(1.),
    Q::Despawn("screen"),
    // car moves into frame, engine sound gets louder
    Q::Tick(2.),
    Q::Tran("car", 700., -50.),
    Q::Paused("car_idle", false),
    Q::Vol("car_idle", 0.),
    Q::Tick(4.),
    Q::Vol("car_idle", 0.3),
    // brake squeak
    Q::Tick(0.5),
    Q::Paused("car_brake", false),
    // car stops
    Q::Tick(0.25),
    Q::Tran("car", -50., -150.),
    // window rolls down
    Q::Tick(1.),
    Q::Paused("car_win_open", false),
    // baby thrown
    Q::Tick(3.5),
    Q::Paused("woosh", false),
    // baby hits ground
    Q::Tick(1.),
    Q::Paused("thump", false),
    // window rolls up
    Q::Tick(1.),
    Q::Paused("car_win_close", false),
    // car turns around
    Q::Tick(4.),
    Q::Flip("car", false),
    // car burnout
    Q::Tick(1.),
    Q::Tran("car", -50., -150.),
    Q::Rot("car", 0.),
    Q::Paused("car_peels_out", false),
    Q::Vol("car_idle", 1.0),
    // car sound fades away
    Q::Tick(0.2),
    Q::Rot("car", 0.7),
    Q::Tick(1.8),
    Q::Tran("car", 700., -50.),
    Q::Vol("car_idle", 0.),
    // somber music plays
    // hold camera for few seconds
    // camera slowly zooms in on baby
    // sudden baby reveal, upbeat wacky music plays
];
// TODO: move these to ANIM_CUE
const CAMERA_ZOOM_IN_START_T: f32 = 29.;
const CAMERA_ZOOM_IN_END_T: f32 = 60.;
const CAMERA_ZOOM_IN_START_S: f32 = 1.;
const CAMERA_ZOOM_IN_END_S: f32 = 0.4;
const CAMERA_ZOOM_IN_START_TR: Vec3 = Vec3::new(0., 0., 0.);
const CAMERA_ZOOM_IN_END_TR: Vec3 = Vec3::new(60., -200., 0.);

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
    flip: Map<Name, Vec<(f32, bool)>>,
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

    fn get_flip(&mut self, name: &Name, time: f32) -> Option<bool> {
        let Some(flips) = self.flip.get(name) else {
            return None;
        };
        let mut flip = None;
        for (t, f) in flips {
            if time >= *t {
                flip = Some(*f);
            }
        }
        flip
    }
}

fn sequence_cues(
    mut names: Query<(Entity, &Name)>,
    audio: Query<&AudioSink>,
    mut sprite: Query<&mut Sprite>,
    mut commands: Commands,
    mut sequence: ResMut<CueSequencer>,
    time: Res<Time>,
    mut dbg: Query<&mut DebugUi>,
) {
    if !sequence.playing {
        return;
    }

    let mut dbg = dbg.single_mut();
    dbg.watch("time", time.elapsed_seconds());

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
        if let Some(flip) = sequence.get_flip(name, t) {
            if let Ok(mut s) = sprite.get_mut(e) {
                s.flip_x = flip;
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
        let mut rot_next = None;
        let mut rot_steps = vec![];
        let mut rot_frames = vec![];

        let mut paused_next = None;
        let mut vol_next = None;
        let mut vol_cues = vec![];
        let mut play_cues = vec![];
        let mut flip_next = None;
        let mut flip_cues = vec![];

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
                Q::Rot(kname, rad) if *kname == name.as_str() => {
                    rot_next = Some(Quat::from_rotation_z(*rad));
                }
                Q::Flip(kname, flip) if *kname == name.as_str() => {
                    flip_next = Some(*flip);
                }
                Q::Tick(dt) => {
                    if let Some(pos_next) = pos_next.take() {
                        pos_frames.push(pos_next);
                        pos_steps.push(t);
                    }
                    if let Some(rot) = rot_next.take() {
                        rot_frames.push(rot);
                        rot_steps.push(t);
                    }
                    if let Some(vol) = vol_next.take() {
                        vol_cues.push((t, vol));
                    }
                    if let Some(paused) = paused_next.take() {
                        play_cues.push((t, paused));
                    }
                    if let Some(flip) = flip_next.take() {
                        flip_cues.push((t, flip));
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
        if let Some(rot) = rot_next {
            rot_frames.push(rot);
            rot_steps.push(t);
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

        if let Some(flip) = flip_next.take() {
            flip_cues.push((t, flip));
        }

        if !(pos_frames.is_empty() && rot_frames.is_empty()) {
            let mut anim = AnimationClip::default();
            if !pos_frames.is_empty() {
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
            }
            if !rot_frames.is_empty() {
                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![name.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: rot_steps,
                        keyframes: Keyframes::Rotation(rot_frames),
                        interpolation: Interpolation::Linear,
                    },
                );
            }

            let mut player = AnimationPlayer::default();
            player.play(animations.add(anim));
            commands.entity(*eid).insert(player);
        }

        if !(vol_cues.is_empty() && play_cues.is_empty()) {
            sequence.audio.insert(name.clone(), (vol_cues, play_cues));
        }

        if !flip_cues.is_empty() {
            sequence.flip.insert(name.clone(), flip_cues);
        }
    }
}

#[derive(Component)]
struct Bezier(CubicSegment<Vec2>);

#[derive(Component)]
struct MainCamera;

// TODO: Make this part of CueSequencer
fn sequence_camera(
    mut camera: Query<(&mut OrthographicProjection, &mut Transform, &Bezier), With<MainCamera>>,
    time: Res<Time>,
) {
    let p1_t = CAMERA_ZOOM_IN_START_T;
    let p2_t = CAMERA_ZOOM_IN_END_T;

    let p1_s = CAMERA_ZOOM_IN_START_S;
    let p2_s = CAMERA_ZOOM_IN_END_S;

    let p1_tr = CAMERA_ZOOM_IN_START_TR;
    let p2_tr = CAMERA_ZOOM_IN_END_TR;

    let t = time.elapsed_seconds();
    let i = inverse_lerp(p1_t..=p2_t, t).unwrap();

    let Ok((mut proj, mut tran, bez)) = camera.get_single_mut() else {
        return;
    };

    let ease = bez.0.ease(i);
    proj.scale = lerp(p1_s..=p2_s, ease);
    tran.translation = p1_tr.lerp(p2_tr, ease);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = Name::new("camera");
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                // When creating our own OrthographicProjection we need to set the far and near
                // values ourselves.
                // See: https://bevy-cheatbook.github.io/2d/camera.html#caveat-nearfar-values
                far: 1000.,
                near: -1000.,
                scaling_mode: ScalingMode::FixedVertical(WINDOW_HEIGHT),
                ..default()
            },
            transform: Transform::from_translation(CAMERA_ZOOM_IN_START_TR),
            ..default()
        },
        MainCamera {},
        camera,
        Bezier(CubicSegment::new_bezier(
            Vec2::new(0.35, 0.), // ease-in-out bezier
            Vec2::new(0.7, 1.),
        )),
    ));

    commands.spawn((
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

    // Pillarboxes
    let pillarbox_h_offset = (WINDOW_WIDTH + PILLARBOX_WIDTH) / 2.;
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(PILLARBOX_WIDTH, WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(pillarbox_h_offset, 0., 100.),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(PILLARBOX_WIDTH, WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(-pillarbox_h_offset, 0., 100.),
        ..default()
    });
}
