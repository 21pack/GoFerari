use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use minifb::{Key, Menu, Window, WindowOptions};

use crossbeam_channel::bounded;

use crate::behaviour::make_step;
use crate::initiator::get_visible_objects;

use ferari::assets;
use ferari::input;
use ferari::render;
use ferari::time;
use ferari::world;
mod behaviour;
mod initiator;

use ferari::render::RenderableEntity;

/// Logical screen width in pixels.
pub const LOGIC_WIDTH: usize = 200;
/// Logical screen height in pixels.
pub const LOGIC_HEIGHT: usize = 200;
/// Tile size in pixels.
pub const TILE_SIZE: usize = 16;
/// Upscaling factor for display.
pub const UPSCALE: usize = 5;

macro_rules! window_main_loop {
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
    let level1_path = project_root.join("examples/input.json");
    let level2_path = project_root.join("examples/input_homka.json");

    let game1 = assets::GameMap::load(level1_path).unwrap();
    let game2 = assets::GameMap::load(level2_path).unwrap();
    let mut game = game1.clone();

    // init draw
    let input_state = Arc::new(input::InputState::new());
    let running = Arc::new(AtomicBool::new(true));
    let (tx_frame, rx_frame) = bounded::<Vec<u32>>(2);

    // framebuffer (`render <-> draw` connection)
    let mut back_buffer: Vec<u32> = vec![0; LOGIC_WIDTH * LOGIC_HEIGHT];

    let mut window = Window::new(
        "Ferari",
        LOGIC_WIDTH * UPSCALE,
        LOGIC_HEIGHT * UPSCALE,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_target_fps(60);

    let mut menu = Menu::new("Edit").unwrap();
    menu.add_item("Exit", 0).shortcut(Key::Escape, 0).build(); // TODO: click don't work

    let mut sub_menu = Menu::new("Levels").unwrap();
    sub_menu.add_item("Level 1", 1).build();
    sub_menu.add_item("Level 2", 2).build();
    sub_menu.add_item("Level 3", 3).build();

    menu.add_sub_menu("Select Level", &sub_menu);
    window.add_menu(&menu);

    // init time
    let mut time = time::Time::new();

    let (mut render, mut camera, mut state) =
        init_level(game, entities_atlas.clone(), tiles_atlas.clone());

    // game loop
    while running.load(Ordering::Acquire) {
        if let Some(id) = window.is_menu_pressed() {
            if id == 1 {
                game = game1.clone();
            } else if id == 2 {
                game = game2.clone();
            } else {
                continue;
            }

            (render, camera, state) = init_level(game, entities_atlas.clone(), tiles_atlas.clone());
        }

        window_main_loop!(window, running, rx_frame, input_state, LOGIC_WIDTH, LOGIC_HEIGHT);

        time.update();

        // test gradient (TODO: move to tests?)
        // let r = ((time.total).sin() * 127.0 + 128.0) as u32;
        // let g = ((time.total + 2.0).sin() * 127.0 + 128.0) as u32;
        // let b = ((time.total + 4.0).sin() * 127.0 + 128.0) as u32;
        // let color = (r << 16) | (g << 8) | b;

        // for px in back_buffer.iter_mut() {
        //     *px = color;
        // }

        // process input
        let input = input_state.read();
        if input.escape {
            running.store(false, Ordering::Release);
        }

        make_step(&mut state, &input, time.delta);

        camera.center_x = state.player.unit.x.floor();
        camera.center_y = state.player.unit.y.floor();

        let units_for_render = get_visible_objects(&state, &camera);

        if units_for_render.is_empty() {
            continue;
        }

        // frame render
        let visible_entities: Vec<RenderableEntity> = units_for_render
            .into_iter()
            .enumerate()
            .map(|(i, unit)| {
                let name_model = if i == 0 { "knight_0" } else { "imp_20" };
                let period = 0.4;
                let cycles = (time.total / period).floor() as u32;
                let animation_num = if cycles.is_multiple_of(2) { "_0" } else { "_1" };
                let full_name = name_model.to_string() + animation_num;

                RenderableEntity::new(unit.x, unit.y, full_name)
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
