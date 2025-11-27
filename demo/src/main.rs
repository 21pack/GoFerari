use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use minifb::{Key, Menu, Window, WindowOptions};

use crossbeam_channel::bounded;

use crate::behaviour::make_step;
use crate::initiator::{get_player_sprite, get_visible_objects};

use ferari::assets;
#[cfg(target_os = "linux")]
use ferari::draw;
use ferari::input;
use ferari::render;
use ferari::time;
use ferari::world;
mod behaviour;
mod initiator;

use ferari::render::RenderableEntity;

/// Animation and movement speedup
pub const MOVEMENT_SPEEDUP: f32 = 1.5;
/// Upscaling factor for display.
pub const UPSCALE: usize = 2;
/// Logical screen width in pixels.
pub const LOGIC_WIDTH: usize = 800 / UPSCALE;
/// Logical screen height in pixels.
pub const LOGIC_HEIGHT: usize = 600 / UPSCALE;
/// Tile size in pixels.
pub const TILE_SIZE: usize = 256;

macro_rules! update_window {
    ($window:ident, $running:ident, $rx_frame:ident, $input_state:ident, $width:ident, $height:ident) => {
        // handle input in this thread
        $input_state.update(&$window);

        if let Ok(frame) = $rx_frame.try_recv() {
            $window.update_with_buffer(&frame, $width, $height).unwrap();
        } else {
            $window.update();
        }

        if $window.is_key_down(Key::Escape) || !$window.is_open() {
            $running.store(false, Ordering::Release);
            break;
        }

        thread::sleep(Duration::from_millis(1));
    };
}

macro_rules! create_menu {
    ($window:ident) => {
        $window.set_target_fps(60);

        let mut menu = Menu::new("Edit").unwrap();
        menu.add_item("Exit", 0).shortcut(Key::Escape, 0).build();

        let mut sub_menu = Menu::new("Levels").unwrap();
        sub_menu.add_item("Level 1", 1).build();
        sub_menu.add_item("Level 2", 2).build();
        sub_menu.add_item("Level 3", 3).build();
        sub_menu.add_item("Level 4", 4).build();

        menu.add_sub_menu("Select Level", &sub_menu);
        $window.add_menu(&menu);
    };
}

fn init_level(
    game: assets::GameMap,
    entities_atlas: assets::Atlas,
    tiles_atlas: assets::Atlas,
) -> (ferari::Render, world::Camera, world::State) {
    // init world_buf
    let world_width = game.size[0] as usize * TILE_SIZE * 2;
    let world_height = game.size[1] as usize * TILE_SIZE * 2;
    let world_buf: Vec<u32> = vec![195213255; world_width * world_height];

    // init render
    let shadow_map: Vec<u8> = vec![0; world_width * world_height];
    let mut render =
        render::Render::new(world_buf, world_height, world_width, entities_atlas, shadow_map);

    // init camera
    let camera = world::Camera::new(
        (world_width / 2) as f32,
        (world_height / 2) as f32,
        LOGIC_WIDTH as u16,
        LOGIC_HEIGHT as u16,
    );

    // init state of game
    let state = world::State::new(&game);

    // prerender
    render.init(&game, &tiles_atlas);

    (render, camera, state)
}

fn main() {
    // Need to find root directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.join("..");

    let assets_path = project_root.join("assets");

    // parse atlases
    let tiles_path = assets_path.join("tiles/atlas.json");
    let entities_path = assets_path.join("entities/atlas.json");

    let tiles_atlas = assets::Atlas::load(tiles_path.to_str().unwrap()).unwrap();
    let entities_atlas = assets::Atlas::load(entities_path.to_str().unwrap()).unwrap();

    // parse game descr
    let level1_path = project_root.join("examples/level1.json");
    let game1 = assets::GameMap::load(level1_path).unwrap();
    let mut game = game1.clone();

    #[cfg(target_os = "linux")]
    {
        let input_state = input_state.clone();
        let running = running.clone();

        thread::spawn(move || {
            draw::run_draw_thread(
                rx_frame,
                input_state,
                running,
                LOGIC_WIDTH,
                LOGIC_HEIGHT,
                UPSCALE,
            );
        });
    }

    #[cfg(target_os = "macos")]
    let mut window = Window::new(
        "Ferari",
        LOGIC_WIDTH * UPSCALE,
        LOGIC_HEIGHT * UPSCALE,
        WindowOptions::default(),
    )
    .unwrap();

    #[cfg(target_os = "macos")]
    create_menu!(window);

    // init draw
    let input_state = Arc::new(input::InputState::new());
    let running = Arc::new(AtomicBool::new(true));
    let (tx_frame, rx_frame) = bounded::<Vec<u32>>(2);

    // framebuffer (`render <-> draw` connection)
    let mut back_buffer: Vec<u32> = vec![0; LOGIC_WIDTH * LOGIC_HEIGHT];

    // init time
    let mut time = time::Time::new();

    let (mut render, mut camera, mut state) =
        init_level(game.clone(), entities_atlas.clone(), tiles_atlas.clone());

    // game loop
    while running.load(Ordering::Acquire) {
        #[cfg(target_os = "macos")]
        if let Some(id) = window.is_menu_pressed() {
            if id == 1 {
                game = game1.clone();
            } else if id == 2 {
                let level2_path = project_root.join("examples/level2.json");
                let game2 = assets::GameMap::load(level2_path).unwrap();
                game = game2.clone();
            } else if id == 3 {
                let level3_path = project_root.join("examples/level3.json");
                let game3 = assets::GameMap::load(level3_path).unwrap();
                game = game3.clone();
            } else if id == 4 {
                let level4_path = project_root.join("examples/level4.json"); // TODO: error in gamemap:280
                let game4 = assets::GameMap::load(level4_path).unwrap();
                game = game4.clone();
            } else {
                continue;
            }

            (render, camera, state) =
                init_level(game.clone(), entities_atlas.clone(), tiles_atlas.clone());
        }

        #[cfg(target_os = "macos")]
        update_window!(window, running, rx_frame, input_state, LOGIC_WIDTH, LOGIC_HEIGHT);

        time.update();

        // process input
        let input = input_state.read();
        if input.escape {
            running.store(false, Ordering::Release);
        }

        make_step(&mut state, &input, time.delta, &game);

        camera.center_x = state.player.unit.pixel_x.floor();
        camera.center_y = state.player.unit.pixel_y.floor();

        let units_for_render = get_visible_objects(&state, &camera);

        if units_for_render.is_empty() {
            continue;
        }

        // frame render
        let visible_entities: Vec<RenderableEntity> = units_for_render
            .into_iter()
            .enumerate()
            .map(|(i, unit)| {
                let sprite_name = if i == 0 {
                    get_player_sprite(&state.player, time.total as f64)
                } else {
                    "box".to_string()
                };

                RenderableEntity::new(unit.pixel_x, unit.pixel_y, sprite_name)
            })
            .collect();

        render.render_frame(&visible_entities, &camera, &mut back_buffer);

        // draw frame
        if tx_frame.try_send(back_buffer.clone()).is_err() {
            // idle
        }

        // fps limit
        thread::sleep(Duration::from_micros(16667)); // ~60 FPS
    }

    println!("Main loop exited");
}
