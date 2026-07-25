use bevy::{
    app::AppExit,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
    utils::HashMap,
};
use std::io;
use std::path::Path;
use std::sync::{mpsc, Mutex};

#[derive(Resource)]
struct CommandReceiver(Mutex<mpsc::Receiver<String>>);

fn main() {
    let max_detected_id = detect_max_texture_id();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        println!("Console ready! Type /help to view commands.");
        loop {
            let mut line = String::new();
            if stdin.read_line(&mut line).is_ok() {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    let _ = tx.send(trimmed);
                }
            }
        }
    });

    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Alloy-Project".into(),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
        )
        .insert_resource(Msaa::Off)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .init_resource::<WorldGrid>()
        .init_resource::<BlockMaterials>()
        .insert_resource(CurrentBlock {
            id: 1,
            max_id: max_detected_id,
        })
        .insert_resource(CommandReceiver(Mutex::new(rx)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_look,
            player_move,
            change_block,
            block_interactions_and_outline,
            grab_mouse,
            update_ui,
            update_fps,
            update_coords,
            handle_console_commands,
        ))
        .run();
}

fn detect_max_texture_id() -> u32 {
    let mut current_id = 1;
    while Path::new(&format!("assets/{}.png", current_id)).exists() {
        current_id += 1;
    }
    (current_id - 1).max(1)
}

#[derive(Resource, Default)]
struct WorldGrid {
    blocks: HashMap<IVec3, Entity>,
}

#[derive(Resource, Default)]
struct BlockMaterials {
    handles: HashMap<u32, Handle<StandardMaterial>>,
}

#[derive(Resource)]
struct BlockMesh(Handle<Mesh>);

#[derive(Resource)]
struct CurrentBlock {
    id: u32,
    max_id: u32,
}

#[derive(Component)]
struct Player {
    velocity: Vec3,
    pitch: f32,
    yaw: f32,
    is_grounded: bool,
    is_flying: bool,
    is_noclip: bool,
    speed_multiplier: f32,
    sensitivity_multiplier: f32,
}

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct Block {
    id: u32,
}

#[derive(Component)]
struct BlockIndicatorUI;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct CoordsText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut block_materials: ResMut<BlockMaterials>,
    mut grid: ResMut<WorldGrid>,
    asset_server: Res<AssetServer>,
    current_block: Res<CurrentBlock>,
) {
    for i in 0..=current_block.max_id {
        let texture_handle = asset_server.load(format!("{}.png", i));
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            unlit: true,
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        });
        block_materials.handles.insert(i, material);
    }

    let box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.insert_resource(BlockMesh(box_mesh.clone()));

    if let Some(ground_material) = block_materials.handles.get(&0) {
        for x in -30..30 {
            for z in -30..30 {
                let pos = IVec3::new(x, 0, z);
                let entity = commands
                    .spawn((
                        PbrBundle {
                            mesh: box_mesh.clone(),
                            material: ground_material.clone(),
                            transform: Transform::from_translation(pos.as_vec3()),
                            ..default()
                        },
                        Block { id: 0 },
                    ))
                    .id();
                grid.blocks.insert(pos, entity);
            }
        }
    }

    commands.spawn((
        Player {
            velocity: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            is_grounded: false,
            is_flying: false,
            is_noclip: false,
            speed_multiplier: 1.0,
            sensitivity_multiplier: 1.0,
        },
        Transform::from_xyz(0.0, 3.0, 0.0),
        GlobalTransform::default(),
        VisibilityBundle::default(),
    )).with_children(|parent| {
        parent.spawn((
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 60.0_f32.to_radians(),
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 1.6, 0.0),
                ..default()
            },
            PlayerCamera,
        ));
    });

    commands.spawn(
        TextBundle::from_section(
            "alloy-project v0.0.6",
            TextStyle { font_size: 16.0, color: Color::WHITE, ..default() },
        ).with_style(Style { position_type: PositionType::Absolute, top: Val::Px(15.0), left: Val::Px(15.0), ..default() })
    );

    // FPS en haut à droite
    commands.spawn((
        TextBundle::from_section(
            "FPS: 0",
            TextStyle { font_size: 16.0, color: Color::WHITE, ..default() },
        ).with_style(Style { position_type: PositionType::Absolute, top: Val::Px(15.0), right: Val::Px(15.0), ..default() }),
        FpsText,
    ));

    // Coordonnées en dessous des FPS (en haut à droite)
    commands.spawn((
        TextBundle::from_section(
            "XYZ: 0.0 / 3.0 / 0.0",
            TextStyle { font_size: 16.0, color: Color::WHITE, ..default() },
        ).with_style(Style { position_type: PositionType::Absolute, top: Val::Px(35.0), right: Val::Px(15.0), ..default() }),
        CoordsText,
    ));

    commands.spawn((
        TextBundle::from_section(
            format!("selected block : {}", current_block.id),
            TextStyle { font_size: 20.0, color: Color::WHITE, ..default() },
        ).with_style(Style { position_type: PositionType::Absolute, bottom: Val::Px(15.0), right: Val::Px(15.0), ..default() }),
        BlockIndicatorUI,
    ));
}

fn handle_console_commands(
    mut commands: Commands,
    receiver: Res<CommandReceiver>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut projection_query: Query<&mut Projection, With<PlayerCamera>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut grid: ResMut<WorldGrid>,
    block_query: Query<(Entity, &Block)>,
    block_materials: Res<BlockMaterials>,
    block_mesh: Res<BlockMesh>,
) {
    if let Ok(rx) = receiver.0.try_lock() {
        for cmd in rx.try_iter() {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() { continue; }

            let (mut player, mut transform) = player_query.single_mut();

            match parts[0] {
                "/fly" => {
                    player.is_flying = true;
                    println!("Fly mode enabled. Use Space to ascend and Shift to descend.");
                }
                "/fall" => {
                    player.is_flying = false;
                    player.is_noclip = false;
                    println!("Gravity mode enabled.");
                }
                "/noclip" => {
                    player.is_noclip = !player.is_noclip;
                    if player.is_noclip {
                        player.is_flying = true;
                    }
                    println!("Noclip set to {}.", player.is_noclip);
                }
                "/tp" => {
                    if parts.len() >= 4 {
                        if let (Ok(x), Ok(y), Ok(z)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                            transform.translation = Vec3::new(x, y, z);
                            player.velocity = Vec3::ZERO;
                            println!("Teleported to ({}, {}, {}).", x, y, z);
                        } else {
                            println!("Invalid coordinates. Example: /tp 0 10 0");
                        }
                    } else {
                        println!("Missing coordinates. Example: /tp 0 10 0");
                    }
                }
                "/spawn" => {
                    transform.translation = Vec3::new(0.0, 3.0, 0.0);
                    player.velocity = Vec3::ZERO;
                    println!("Teleported to spawn (0, 3, 0).");
                }
                "/reset" => {
                    for (entity, _) in block_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    grid.blocks.clear();

                    if let Some(ground_material) = block_materials.handles.get(&0) {
                        for x in -30..30 {
                            for z in -30..30 {
                                let pos = IVec3::new(x, 0, z);
                                let entity = commands
                                    .spawn((
                                        PbrBundle {
                                            mesh: block_mesh.0.clone(),
                                            material: ground_material.clone(),
                                            transform: Transform::from_translation(pos.as_vec3()),
                                            ..default()
                                        },
                                        Block { id: 0 },
                                    ))
                                    .id();
                                grid.blocks.insert(pos, entity);
                            }
                        }
                    }

                    transform.translation = Vec3::new(0.0, 3.0, 0.0);
                    player.velocity = Vec3::ZERO;
                    player.pitch = 0.0;
                    player.yaw = 0.0;
                    
                    println!("World and player reset.");
                }
                "/save" => {
                    let filename = if parts.len() > 1 { parts[1] } else { "world.txt" };
                    let _ = std::fs::create_dir_all("grids");
                    let filepath = format!("grids/{}", filename);
                    let mut lines = Vec::new();
                    for (pos, &entity) in grid.blocks.iter() {
                        if let Ok((_, block)) = block_query.get(entity) {
                            lines.push(format!("{} {} {} {}", pos.x, pos.y, pos.z, block.id));
                        }
                    }
                    if std::fs::write(&filepath, lines.join("\n")).is_ok() {
                        println!("World saved to {}.", filepath);
                    } else {
                        println!("Failed to save world to {}.", filepath);
                    }
                }
                "/load" => {
                    let filename = if parts.len() > 1 { parts[1] } else { "world.txt" };
                    let filepath = format!("grids/{}", filename);
                    if let Ok(data) = std::fs::read_to_string(&filepath) {
                        for (entity, _) in block_query.iter() {
                            commands.entity(entity).despawn();
                        }
                        grid.blocks.clear();

                        for line in data.lines() {
                            let tokens: Vec<&str> = line.split_whitespace().collect();
                            if tokens.len() == 4 {
                                if let (Ok(x), Ok(y), Ok(z), Ok(block_id)) = (
                                    tokens[0].parse::<i32>(),
                                    tokens[1].parse::<i32>(),
                                    tokens[2].parse::<i32>(),
                                    tokens[3].parse::<u32>(),
                                ) {
                                    let pos = IVec3::new(x, y, z);
                                    if let Some(mat) = block_materials.handles.get(&block_id) {
                                        let entity = commands.spawn((
                                            PbrBundle {
                                                mesh: block_mesh.0.clone(),
                                                material: mat.clone(),
                                                transform: Transform::from_translation(pos.as_vec3()),
                                                ..default()
                                            },
                                            Block { id: block_id },
                                        )).id();
                                        grid.blocks.insert(pos, entity);
                                    }
                                }
                            }
                        }
                        println!("World loaded from {}.", filepath);
                    } else {
                        println!("Failed to load world from {}.", filepath);
                    }
                }
                "/list" => {
                    println!("\n--- Saved Worlds ---");
                    if let Ok(entries) = std::fs::read_dir("grids") {
                        let mut found = false;
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_file() {
                                    println!("- {}", entry.file_name().to_string_lossy());
                                    found = true;
                                }
                            }
                        }
                        if !found {
                            println!("No saved worlds found.");
                        }
                    } else {
                        println!("The grids directory does not exist or is empty.");
                    }
                    println!("--------------------\n");
                }
                "/sensivity" | "/sensitivity" => {
                    if parts.len() > 1 {
                        if let Ok(val) = parts[1].parse::<f32>() {
                            player.sensitivity_multiplier = val;
                            println!("Sensitivity set to x{}", val);
                        } else {
                            println!("Invalid value. Example: /sensivity 1.5");
                        }
                    } else {
                        println!("Missing multiplier. Example: /sensivity 1.5");
                    }
                }
                "/speed" => {
                    if parts.len() > 1 {
                        if let Ok(val) = parts[1].parse::<f32>() {
                            player.speed_multiplier = val;
                            println!("Speed set to x{}", val);
                        } else {
                            println!("Invalid value. Example: /speed 2.0");
                        }
                    } else {
                        println!("Missing multiplier. Example: /speed 2.0");
                    }
                }
                "/fov" => {
                    if parts.len() > 1 {
                        if let Ok(val) = parts[1].parse::<f32>() {
                            if let Ok(mut projection) = projection_query.get_single_mut() {
                                if let Projection::Perspective(ref mut perspective) = *projection {
                                    perspective.fov = val.to_radians();
                                    println!("FOV set to {} degrees.", val);
                                }
                            }
                        } else {
                            println!("Invalid value. Example: /fov 60");
                        }
                    } else {
                        println!("Missing FOV value. Example: /fov 60");
                    }
                }
                "/stop" => {
                    println!("Stopping game...");
                    app_exit_events.send(AppExit::Success);
                }
                "/help" => {
                    println!("\n--- Available Commands ---");
                    println!("/fly                - Enables flight mode");
                    println!("/fall               - Disables flight mode (enables gravity)");
                    println!("/noclip             - Toggles noclip mode");
                    println!("/tp <x> <y> <z>     - Teleports player to coordinates");
                    println!("/spawn              - Teleports player to spawn point");
                    println!("/reset              - Resets world and player state");
                    println!("/save [filename]    - Saves the world to the grids folder");
                    println!("/load [filename]    - Loads a world from the grids folder");
                    println!("/list               - Lists all saved worlds in the grids folder");
                    println!("/sensivity <number> - Adjusts mouse sensitivity (default: 1.0)");
                    println!("/speed <number>     - Adjusts movement speed (default: 1.0)");
                    println!("/fov <degrees>      - Adjusts field of view (default: 60)");
                    println!("/stop               - Closes the game");
                    println!("/help               - Displays this help message");
                    println!("---------------------------\n");
                }
                _ => {
                    println!("Unknown command: {}. Type /help for options.", parts[0]);
                }
            }
        }
    }
}

fn update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    mut last_shown: Local<i32>,
) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) else { return; };
    let Some(value) = fps.smoothed() else { return; };
    let rounded = value.round() as i32;
    if rounded != *last_shown {
        *last_shown = rounded;
        for mut text in &mut query {
            text.sections[0].value = format!("FPS: {}", rounded);
        }
    }
}

fn update_coords(
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<&mut Text, With<CoordsText>>,
) {
    if let Ok(transform) = player_query.get_single() {
        let pos = transform.translation;
        for mut text in &mut query {
            text.sections[0].value = format!("XYZ: {:.1} / {:.1} / {:.1}", pos.x, pos.y, pos.z);
        }
    }
}

fn grab_mouse(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();
    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }
}

fn player_look(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut mouse_motion: EventReader<MouseMotion>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    if window.cursor.grab_mode == CursorGrabMode::None { return; }

    let (mut player, mut player_transform) = player_query.single_mut();
    let mut cam_transform = cam_query.single_mut();

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    let sensitivity = 0.002 * player.sensitivity_multiplier;
    player.yaw -= delta.x * sensitivity;
    player.pitch -= delta.y * sensitivity;
    player.pitch = player.pitch.clamp(-1.54, 1.54);

    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, player.yaw);
    cam_transform.rotation = Quat::from_axis_angle(Vec3::X, player.pitch);
}

fn check_collision(pos: Vec3, grid: &WorldGrid) -> bool {
    let min = (pos + Vec3::new(-0.3, 0.0, -0.3)).round().as_ivec3();
    let max = (pos + Vec3::new(0.3, 1.9, 0.3)).round().as_ivec3();

    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                if grid.blocks.contains_key(&IVec3::new(x, y, z)) {
                    return true;
                }
            }
        }
    }
    false
}

fn intersects_player(voxel: IVec3, player_feet: Vec3) -> bool {
    let player_min = player_feet + Vec3::new(-0.3, 0.0, -0.3);
    let player_max = player_feet + Vec3::new(0.3, 2.0, 0.3);

    let voxel_min = voxel.as_vec3() - Vec3::splat(0.5);
    let voxel_max = voxel.as_vec3() + Vec3::splat(0.5);

    player_min.x < voxel_max.x && player_max.x > voxel_min.x &&
    player_min.y < voxel_max.y && player_max.y > voxel_min.y &&
    player_min.z < voxel_max.z && player_max.z > voxel_min.z
}

fn player_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    grid: Res<WorldGrid>,
) {
    let (mut player, mut transform) = player_query.single_mut();
    
    let forward = transform.rotation * Vec3::NEG_Z;
    let right = transform.rotation * Vec3::X;
    
    let mut input_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { input_dir += forward; }
    if keys.pressed(KeyCode::KeyS) { input_dir -= forward; }
    if keys.pressed(KeyCode::KeyA) { input_dir -= right; }
    if keys.pressed(KeyCode::KeyD) { input_dir += right; }

    let speed = 6.0 * player.speed_multiplier;
    let jump_force = 8.0;
    let gravity = 24.0;
    let friction = 12.0;
    let dt = time.delta_seconds();

    let target_velocity = input_dir.normalize_or_zero() * speed;

    player.velocity.x += (target_velocity.x - player.velocity.x) * friction * dt;
    player.velocity.z += (target_velocity.z - player.velocity.z) * friction * dt;
    
    if player.is_flying || player.is_noclip {
        player.velocity.y = 0.0;
        if keys.pressed(KeyCode::Space) { player.velocity.y = speed; }
        if keys.pressed(KeyCode::ShiftLeft) { player.velocity.y = -speed; }
    } else {
        player.velocity.y -= gravity * dt;
    }

    let mut new_pos = transform.translation;

    if player.is_noclip {
        new_pos += player.velocity * dt;
    } else {
        if player.velocity.x.abs() > 0.01 {
            new_pos.x += player.velocity.x * dt;
            if check_collision(new_pos, &grid) {
                new_pos.x = transform.translation.x; 
                player.velocity.x = 0.0;
            }
        }

        if player.velocity.z.abs() > 0.01 {
            new_pos.z += player.velocity.z * dt;
            if check_collision(new_pos, &grid) {
                new_pos.z = transform.translation.z; 
                player.velocity.z = 0.0;
            }
        }

        new_pos.y += player.velocity.y * dt;
        if check_collision(new_pos, &grid) {
            new_pos.y = transform.translation.y;
            if !player.is_flying && player.velocity.y < 0.0 {
                player.is_grounded = true;
            }
            player.velocity.y = 0.0;
        } else {
            player.is_grounded = false;
        }

        if !player.is_flying && player.is_grounded && keys.just_pressed(KeyCode::Space) {
            player.velocity.y = jump_force;
            player.is_grounded = false;
        }
    }

    transform.translation = new_pos;
}

fn change_block(
    mut scroll_evr: EventReader<MouseWheel>,
    mut current_block: ResMut<CurrentBlock>,
) {
    for ev in scroll_evr.read() {
        if ev.y < 0.0 {
            current_block.id = if current_block.id < current_block.max_id { current_block.id + 1 } else { 0 };
        } else if ev.y > 0.0 {
            current_block.id = if current_block.id > 0 { current_block.id - 1 } else { current_block.max_id };
        }
    }
}

fn update_ui(
    current_block: Res<CurrentBlock>,
    mut query: Query<&mut Text, With<BlockIndicatorUI>>,
) {
    if current_block.is_changed() {
        for mut text in &mut query {
            text.sections[0].value = format!("selected block : {}", current_block.id);
        }
    }
}

fn raycast_voxels(origin: Vec3, dir: Vec3, max_distance: f32, grid: &WorldGrid) -> Option<(IVec3, IVec3)> {

    let origin = origin + Vec3::splat(0.5);
    let dir = dir.normalize();

    let mut voxel = origin.floor().as_ivec3();
    let mut prev_voxel = voxel;

    let step = IVec3::new(
        if dir.x > 0.0 { 1 } else { -1 },
        if dir.y > 0.0 { 1 } else { -1 },
        if dir.z > 0.0 { 1 } else { -1 },
    );

    let t_delta = Vec3::new(
        if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f32::INFINITY },
        if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f32::INFINITY },
        if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f32::INFINITY },
    );

    let mut t_max = Vec3::new(
        if dir.x != 0.0 { ((voxel.x as f32 + if step.x > 0 { 1.0 } else { 0.0 }) - origin.x) / dir.x } else { f32::INFINITY },
        if dir.y != 0.0 { ((voxel.y as f32 + if step.y > 0 { 1.0 } else { 0.0 }) - origin.y) / dir.y } else { f32::INFINITY },
        if dir.z != 0.0 { ((voxel.z as f32 + if step.z > 0 { 1.0 } else { 0.0 }) - origin.z) / dir.z } else { f32::INFINITY },
    );

    let mut travelled = 0.0_f32;
    while travelled <= max_distance {
        if grid.blocks.contains_key(&voxel) {
            return Some((voxel, prev_voxel));
        }
        prev_voxel = voxel;

        if t_max.x < t_max.y && t_max.x < t_max.z {
            voxel.x += step.x;
            travelled = t_max.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            voxel.y += step.y;
            travelled = t_max.y;
            t_max.y += t_delta.y;
        } else {
            voxel.z += step.z;
            travelled = t_max.z;
            t_max.z += t_delta.z;
        }
    }
    None
}

fn block_interactions_and_outline(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    player_query: Query<&Transform, With<Player>>,
    cam_query: Query<&GlobalTransform, With<PlayerCamera>>,
    mut grid: ResMut<WorldGrid>,
    block_mesh: Res<BlockMesh>,
    block_materials: Res<BlockMaterials>,
    current_block: Res<CurrentBlock>,
    mut gizmos: Gizmos,
) {
    let window = windows.single();
    if window.cursor.grab_mode == CursorGrabMode::None { return; }

    let player_feet = player_query.single().translation;
    let cam_transform = cam_query.single().compute_transform();

    let forward = cam_transform.rotation * Vec3::NEG_Z;
    let max_distance = 6.0;

    let break_block = mouse.just_pressed(MouseButton::Left);
    let place_block = mouse.just_pressed(MouseButton::Right);

    let Some((hit_voxel, prev_voxel)) = raycast_voxels(cam_transform.translation, forward, max_distance, &grid) else {
        return;
    };

    gizmos.cuboid(
        Transform::from_translation(hit_voxel.as_vec3()).with_scale(Vec3::splat(1.02)),
        Color::WHITE,
    );

    if break_block {
        if let Some(entity) = grid.blocks.remove(&hit_voxel) {
            commands.entity(entity).despawn();
        }
    } else if place_block && !grid.blocks.contains_key(&prev_voxel) {
        if !intersects_player(prev_voxel, player_feet) {
            if let Some(material) = block_materials.handles.get(&current_block.id) {
                let new_entity = commands.spawn((
                    PbrBundle {
                        mesh: block_mesh.0.clone(),
                        material: material.clone(),
                        transform: Transform::from_translation(prev_voxel.as_vec3()),
                        ..default()
                    },
                    Block { id: current_block.id },
                )).id();
                grid.blocks.insert(prev_voxel, new_entity);
            }
        }
    }
}
