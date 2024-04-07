use std::collections::HashMap as Map;
use std::f32::consts::PI;

use bevy::{
    app::AppExit,
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    window::PrimaryWindow,
};

#[derive(Component)]
struct Control;
#[derive(Component, Clone, Copy, Default, Debug)]
enum Collide {
    #[default]
    Square,
    StepL,
    StepR,
    SlopeL,
    SlopeR,
}
#[derive(Resource)]
struct PhysicsTick(f32);
#[derive(Component, Default)]
struct Movement {
    ctl: Vec2,
    force: Vec2,
    out: Vec2,
    climb: bool,
}
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug)]
struct Tile(u8);
#[derive(Event)]
struct Quit; // custom quit event used to save map before actual AppExit
#[derive(Resource, Default, Deref)]
struct TileTypes(Vec<(Color, Collide, Option<(Handle<Image>, (f32, f32), f32)>)>);

#[derive(Component, Default)]
struct DebugUi {
    text: Map<&'static str, String>,
    collisions: Vec<Aabb2d>,
    ctl_aabb: Option<Aabb2d>,
    cursor: Vec2,
}

impl DebugUi {
    fn watch(&mut self, key: &'static str, val: impl std::fmt::Debug) {
        self.text.insert(key, format!("{:?}", val));
    }
}
#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Baby".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(PhysicsTick(0.))
        .insert_resource(TileTypes(vec![default()]))
        .add_event::<Quit>()
        .add_systems(Startup, setup_graphics)
        .add_systems(
            Update,
            (check_kbd, check_collide, update_movement, update_camera).chain(),
        )
        .add_systems(Update, (check_mouse, on_quit))
        .add_systems(PostUpdate, draw_debug)
        .run();
}

const TILE_SZ: f32 = 50.;
const MAP: (Vec2, usize, [u8; 8 * 8]) = (
    Vec2::new(-200., -200.),
    8,
    [
        1, 1, 1, 1, 1, 1, 1, 1, // 1
        1, 0, 0, 0, 0, 0, 0, 1, // 2
        1, 0, 0, 0, 0, 0, 0, 1, // 3
        1, 0, 0, 0, 0, 0, 0, 1, // 4
        1, 0, 0, 0, 0, 1, 0, 1, // 5
        1, 0, 0, 0, 0, 0, 0, 1, // 6
        1, 0, 0, 0, 0, 1, 0, 1, // 7
        1, 1, 1, 1, 1, 1, 1, 1, // 8
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

fn setup_graphics(
    mut command: Commands,
    assets: Res<AssetServer>,
    mut win: Query<&mut Window, With<PrimaryWindow>>,
    mut tile_types: ResMut<TileTypes>,
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

    command.spawn((
        Control,
        Movement::default(),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(0.8, 1.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::new(45., 45., 1.),
                ..default()
            },
            texture: assets.load("baby.png"),
            ..default()
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

fn check_kbd(
    kbd: Res<ButtonInput<KeyCode>>,
    mut quit: EventWriter<Quit>,
    mut ctl: Query<&mut Movement, With<Control>>,
) {
    if kbd.pressed(KeyCode::Escape) {
        quit.send(Quit);
    }

    let mut vx = 0.;
    let mut vy = 0.;
    if kbd.pressed(KeyCode::ArrowLeft) {
        vx -= 1.;
    }
    if kbd.pressed(KeyCode::ArrowRight) {
        vx += 1.;
    }
    if kbd.pressed(KeyCode::ArrowUp) {
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

fn check_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    win: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
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
    mut dbg: Query<&mut DebugUi>,
) {
    let (cam, cam_trans) = cam.single();
    let Some(cursor) = win.single().cursor_position() else {
        return;
    };
    let Some(cursor) = cam.viewport_to_world_2d(cam_trans, cursor) else {
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

            tile.0 = (tile.0 + 1) % (tile_types.len() as u8);
            if tile.0 == 0 {
                commands.get_entity(e).unwrap().despawn();
            } else {
                s.color = tile_types.0[tile.0 as usize].0;
                if let Some((hndl, _, _)) = &tile_types.0[tile.0 as usize].2 {
                    *img = hndl.clone();
                } else {
                    *img = default();
                }
                *col = tile_types.0[tile.0 as usize].1;
            }
            return;
        }

        // no tile, need to insert
        let tile_pos = (cursor / TILE_SZ).round() * TILE_SZ;
        Tile::spawn(&mut commands, 1, tile_pos.extend(0.), &tile_types);
    }
}

fn collide_push(aabb: &Aabb2d, col: &Collide, col_aabb: &Aabb2d) -> (Vec2, bool, bool) {
    let lt = col_aabb.min.x - aabb.max.x;
    let rt = col_aabb.max.x - aabb.min.x;
    let up = col_aabb.max.y - aabb.min.y;
    let dn = col_aabb.min.y - aabb.max.y;
    let horz = if lt.abs() < rt.abs() { lt } else { rt };
    let vert = if dn.abs() < up.abs() { dn } else { up };
    match col {
        Collide::Square => {
            if horz.abs() > vert.abs() {
                (Vec2::new(0., vert), false, true)
            } else {
                (Vec2::new(horz, 0.), true, false)
            }
        }
        Collide::StepL | Collide::SlopeL => {
            // collide like a triangle |\
            use std::f32::consts::FRAC_1_SQRT_2;
            let n = Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
            let a = col_aabb.center();
            let p = aabb.min;
            if (p - a).y > -(p - a).x {
                return (Vec2::ZERO, false, false);
            }
            let dist = (p - a - ((p - a).dot(n) * n)).length();
            let dampv = matches!(col, Collide::StepL);
            match (
                horz.abs() < dist,
                horz.abs() < vert.abs(),
                vert.abs() < dist,
            ) {
                (true, true, _) => (Vec2::new(horz, 0.), true, false),
                (_, false, true) => (Vec2::new(0., vert), false, true),
                _ => (Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2) * dist, false, dampv),
            }
        }
        Collide::StepR | Collide::SlopeR => {
            // collide like a triangel /|
            use std::f32::consts::FRAC_1_SQRT_2;
            let n = Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
            let a = col_aabb.center();
            let p = Vec2::new(aabb.max.x, aabb.min.y);
            if (p - a).y > (p - a).x {
                return (Vec2::ZERO, false, false);
            }
            let dist = (p - a - ((p - a).dot(n) * n)).length();
            let dampv = matches!(col, Collide::StepR);
            match (
                horz.abs() < dist,
                horz.abs() < vert.abs(),
                vert.abs() < dist,
            ) {
                (true, true, _) => (Vec2::new(horz, 0.), true, false),
                (_, false, true) => (Vec2::new(0., vert), false, true),
                _ => (
                    Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2) * dist,
                    false,
                    dampv,
                ),
            }
        }
    }
}

// the intent is to cast the ctl's aabb along ctl's velocity and check for any collisions
// if there are any collisions, then reduce velocity until there aren't
//
// this is not working correctly as it sees collisions where it shouldn't
fn check_collide(
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

        // sort bottom-to-top, left-to-right
        // collisions.sort_by(|c1, c2| {
        //     (c2.min.y.total_cmp(&c1.min.y)).then(c1.min.x.total_cmp(&c2.min.x))
        // });

        // sort by distance to aabb
        collisions.sort_by(|c1, c2| c1.0.total_cmp(&c2.0));

        for (_, col, col_aabb) in &collisions {
            if !aabb.intersects(col_aabb) {
                continue;
            }
            let (push, damph, dampv) = collide_push(&aabb, col, col_aabb);
            if push == Vec2::ZERO {
                continue;
            }

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
        }
        dt -= 1.;
    }
    if !collisions.is_empty() {
        let mut dbg = dbg.single_mut();
        dbg.watch("vctl", v.ctl);
        dbg.watch("vforce", v.force);
        dbg.watch("pos", t.translation);
        dbg.watch("rot", t.rotation.to_axis_angle());
        dbg.watch("climb", v.climb);
        dbg.collisions = collisions.into_iter().map(|(_, _, c)| c).collect();
        dbg.ctl_aabb = Some(aabb);
    }

    let tnew = aabb.center();
    v.out = tnew - t.translation.xy();
    if dt != update_rem.0 {
        update_rem.0 = dt;
    }
}

fn update_movement(mut movers: Query<(&mut Transform, &Movement, &mut Sprite)>) {
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

fn update_camera(
    mut trans: Query<&mut Transform>,
    cam: Query<Entity, With<Camera>>,
    ctl: Query<Entity, With<Control>>,
) {
    let ctl = ctl.single();
    let ctl = trans.get(ctl).unwrap().translation;

    let cam = cam.single();
    let mut cam = trans.get_mut(cam).unwrap();
    let cam_bound = 100.;
    if (ctl.x - cam.translation.x).abs() > cam_bound {
        let dx = ctl.x - cam.translation.x;
        if dx < 0. {
            cam.translation.x += dx + cam_bound;
        } else {
            cam.translation.x += dx - cam_bound;
        }
    }
}

fn draw_debug(mut gizmos: Gizmos, mut dbg: Query<(&mut Text, &DebugUi)>) {
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
        for (i, aabb) in dbg.collisions.iter().enumerate() {
            gizmos.rect_2d(
                aabb.center(),
                0.,
                aabb.half_size() * 2.,
                Color::rgb(1. - (i as f32 * n), 0., 0.),
            );
            // gizmos.ray_2d(*origin, *ray, Color::GREEN);
        }
        if let Some(aabb) = &dbg.ctl_aabb {
            gizmos.rect_2d(aabb.center(), 0., aabb.half_size() * 2., Color::GREEN);
        }
    }
    let cursor = (dbg.cursor / TILE_SZ).round() * TILE_SZ;
    gizmos.rect_2d(cursor, 0., Vec2::new(TILE_SZ, TILE_SZ), Color::GREEN);
}

fn on_quit(
    quit: EventReader<Quit>,
    tiles: Query<(&Transform, &Tile), With<Tile>>,
    mut exit: EventWriter<AppExit>,
) {
    if !quit.is_empty() {
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
        for (y, row) in map.into_iter().rev().enumerate() {
            print!("    ");
            for t in row {
                print!("{t}, ");
            }
            println!(" // {y}");
        }
        println!("  ],");
        println!(");");

        exit.send(AppExit);
    }
}
