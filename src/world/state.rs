use crate::assets::GameMap;

/// Represents the current game state containing all units.
///
/// The `State` struct manages the player unit and all mob units in the game,
/// tracking their positions and movement speeds for game simulation.
#[derive(Debug, Default)]
pub struct State {
    /// The player-controlled unit
    pub player: Player,

    /// Collection of all non-player mobile units
    pub mobs: Vec<Unit>,

    /// 1D vector representing the 2D map. Stores the index of the mob
    /// within `self.mobs` that currently occupies the tile, or `None`
    pub mob_grid: Vec<Option<usize>>,

    /// Width of the game map (in tiles), used for calculating 1D index from 2D coordinates
    pub grid_width: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    NE, // North East (Up)
    SE, // South East (Right)
    SW, // South West (Down)
    NW, // North West (Left)
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::NE => "ne",
            Direction::SE => "se",
            Direction::SW => "sw",
            Direction::NW => "nw",
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::SE
    }
}

/// State for discrete movement and animation.
#[derive(Debug, Clone)]
pub enum UnitMovement {
    /// Standing still
    Idle,

    /// Moving from one tile to another
    Moving {
        /// Initial position
        start_x: f32,
        start_y: f32,

        /// Target position
        target_x: f32,
        target_y: f32,

        /// Movement progress since starting
        elapsed_time: f32,
        /// Total movement time
        duration: f32,
    },

    /// Pushing box from one tile to another. same as `Moving`, but with different animation
    Pushing {
        /// Initial position
        start_x: f32,
        start_y: f32,

        /// Target position
        target_x: f32,
        target_y: f32,

        /// Movement progress since starting
        elapsed_time: f32,
        /// Total movement time
        duration: f32,
    },
}

impl Default for UnitMovement {
    fn default() -> Self {
        Self::Idle
    }
}

/// Represents a unit entity in the game world with position and movement capabilities.
///
/// Units can be either player-controlled or game-controlled mobs. Each unit has
/// a position in 2D space and speed components for movement simulation.
#[derive(Debug, Default, Clone)]
pub struct Unit {
    /// Pixel coordinates (for animation)
    pub pixel_x: f32,
    pub pixel_y: f32,

    /// Logical coordinates (tile-based)
    pub tile_x: i32,
    pub tile_y: i32,

    /// Horizontal movement speed
    pub x_speed: f32,
    /// Vertical movement speed
    pub y_speed: f32,

    /// State for movement
    pub movement: UnitMovement,
    /// Current facing direction
    pub direction: Direction,
}

impl Unit {
    /// Creates a new `Unit` with the specified position and movement parameters.
    ///
    /// # Arguments
    ///
    /// * `pixel_x` - Initial X-coordinate position
    /// * `pixel_x` - Initial Y-coordinate position
    /// * `tile_x` - Initial tile-based X-coordinate position
    /// * `tile_y` - Initial tile-based Y-coordinate position
    /// * `x_speed` - Initial horizontal movement speed
    /// * `y_speed` - Initial vertical movement speed
    ///
    /// # Returns
    ///
    /// A new `Unit` instance with the specified properties.
    #[allow(dead_code)]
    pub fn new(
        pixel_x: f32,
        pixel_y: f32,
        tile_x: i32,
        tile_y: i32,
        x_speed: f32,
        y_speed: f32,
    ) -> Self {
        Self { pixel_x, pixel_y, tile_x, tile_y, x_speed, y_speed, ..Default::default() }
    }
}

/// Represents the player's state. Primarily wraps a `Unit`.
#[derive(Debug, Default)]
pub struct Player {
    /// Player's data
    pub unit: Unit,
}

impl Player {
    pub fn new(unit: Unit) -> Self {
        Self { unit: unit }
    }
}

/// Converts tile coordinates to pixel coordinates of the tile center
/// in the world buffer coordinate system.
fn tile_to_world_buf_pos(
    tile_x: i32,
    tile_y: i32,
    tile_size: u32,
    world_width: u32,
    world_height: u32,
) -> (f32, f32) {
    let ts = tile_size as i32;
    let ww = world_width as i32;
    let wh = world_height as i32;

    let offset_x = ww / 2;
    let offset_y = wh / 2 - ts;

    let x = (tile_x - tile_y) * (ts / 2) + offset_x;
    let y = (tile_x + tile_y) * (ts / 4) + offset_y - (ts / 2);

    let todo_remove_me = ts / 8;

    let center_x = x + ts / 2;
    let center_y = y + ts / 4 + todo_remove_me;

    (center_x as f32, center_y as f32)
}

impl State {
    /// Creates a new `State` by getting unit data from a `GameMap`.
    ///
    /// This constructor processes all units defined in the game map, identifying
    /// the player unit and initializing all mob units with their starting positions
    /// and movement behaviors.
    ///
    /// # Arguments
    ///
    /// * `game_map` - Reference to the `GameMap` containing unit definitions
    ///
    /// # Returns
    ///
    /// A new `State` instance with:
    /// - Player unit initialized from the mob marked as `is_player`
    /// - All other mobs initialized with their respective behaviors and speeds
    ///
    /// # Behavior
    ///
    /// - The player unit is given fixed movement speeds (10.0 in both directions)
    /// - Mob units derive their movement from behavior definitions:
    ///   - "right": positive x_speed
    ///   - "left": negative x_speed  
    ///   - "up": negative y_speed
    ///   - "down": positive y_speed
    ///   - Mobs without behavior definitions get zero movement speed
    /// - Mobs without specified speed default to 0.0
    pub fn new(game_map: &GameMap) -> Self {
        let mut player: Option<Unit> = None;
        let mut mobs: Vec<Unit> = Vec::new();

        let world_width = game_map.size[0] * game_map.tile_size * 2;
        let world_height = game_map.size[1] * game_map.tile_size * 2;

        for mob_data in game_map.iter_mobs() {
            let (world_x, world_y) = tile_to_world_buf_pos(
                mob_data.x_start as i32,
                mob_data.y_start as i32,
                game_map.tile_size,
                world_width,
                world_height,
            );

            let mut unit = Unit {
                pixel_x: world_x,
                pixel_y: world_y,
                tile_x: mob_data.x_start as i32,
                tile_y: mob_data.y_start as i32,
                ..Default::default()
            };

            if mob_data.is_player {
                unit.x_speed = 10.0;
                unit.y_speed = 10.0;
                player = Some(unit);
                continue;
            }

            if let Some(beh) = &mob_data.behaviour {
                let mob_direction = beh.direction.as_deref().unwrap_or("none");
                let mob_speed = beh.speed.unwrap_or(0.0);

                match mob_direction {
                    "right" => unit.x_speed = mob_speed,
                    "left" => unit.x_speed = -mob_speed,
                    "up" => unit.y_speed = -mob_speed,
                    "down" => unit.y_speed = mob_speed,
                    _ => {}
                }
            }

            mobs.push(unit);
        }

        let width = game_map.size[0] as usize;
        let height = game_map.size[1] as usize;
        let grid_width = width as i32;

        let mut mob_grid = vec![None; width * height];

        for (i, unit) in mobs.iter().enumerate() {
            let idx = (unit.tile_y * grid_width + unit.tile_x) as usize;
            mob_grid[idx] = Some(i);
        }

        Self { player: Player::new(player.unwrap()), mobs, mob_grid, grid_width }
    }

    /// Updates the `mob_grid` to reflect a mob's movement from one tile to another.
    ///
    /// # Arguments
    ///
    /// * `mob_index` - The index of the mob being moved within `self.mobs`.
    /// * `old_x` - The tile X-coordinate of the mob's starting position.
    /// * `old_y` - The tile Y-coordinate of the mob's starting position.
    /// * `new_x` - The tile X-coordinate of the mob's destination position.
    /// * `new_y` - The tile Y-coordinate of the mob's destination position.
    pub fn update_mob_pos(
        &mut self,
        mob_index: usize,
        old_x: i32,
        old_y: i32,
        new_x: i32,
        new_y: i32,
    ) {
        let old_idx = (old_y * self.grid_width + old_x) as usize;
        let new_idx = (new_y * self.grid_width + new_x) as usize;

        if let Some(current_idx) = self.mob_grid.get(old_idx) {
            if *current_idx == Some(mob_index) {
                self.mob_grid[old_idx] = None;
            }
        }

        if new_idx < self.mob_grid.len() {
            self.mob_grid[new_idx] = Some(mob_index);
        }
    }

    /// Retrieves the index of the mob occupying the given tile coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - Target tile X-coordinate
    /// * `y` - Target tile Y-coordinate
    ///
    /// # Returns
    ///
    /// `Some(index)` if a mob is present at `(x, y)`.
    /// Returns `None` if the tile is empty or if the provided
    /// coordinates are out of the map bounds.
    pub fn get_mob_at(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }

        let idx = (y * self.grid_width + x) as usize;
        if idx < self.mob_grid.len() {
            self.mob_grid[idx]
        } else {
            None
        }
    }
}

// #[cfg(test)]
// mod state_tests {
//     use crate::assets::{Behaviour, BehaviourType, GameMap, Mob, TileType};
//     use crate::world::State;

//     fn make_test_map() -> GameMap {
//         let mut mobs = std::collections::HashMap::new();

//         mobs.insert(
//             "player".to_string(),
//             Mob {
//                 name: "player".to_string(),
//                 x_start: 0,
//                 y_start: 0,
//                 asset: "knight".to_string(),
//                 is_player: true,
//                 behaviour: None,
//             },
//         );

//         mobs.insert(
//             "mob_right".to_string(),
//             Mob {
//                 name: "mob_right".to_string(),
//                 x_start: 10,
//                 y_start: 0,
//                 asset: "imp".to_string(),
//                 is_player: false,
//                 behaviour: Some(Behaviour {
//                     behaviour_type: BehaviourType::Walker,
//                     direction: Some("right".to_string()),
//                     speed: Some(1.0),
//                 }),
//             },
//         );

//         mobs.insert(
//             "mob_up".to_string(),
//             Mob {
//                 name: "mob_up".to_string(),
//                 x_start: 0,
//                 y_start: 10,
//                 asset: "ghost".to_string(),
//                 is_player: false,
//                 behaviour: Some(Behaviour {
//                     behaviour_type: BehaviourType::Walker,
//                     direction: Some("up".to_string()),
//                     speed: Some(0.5),
//                 }),
//             },
//         );

//         GameMap {
//             name: "test_map".to_string(),
//             tile_size: 16,
//             size: [5, 5],
//             mobs,
//             objects: std::collections::HashMap::new(),
//             tiles: std::collections::HashMap::new(),
//             walk_map: vec![TileType::Empty; 25],
//         }
//     }

//     #[test]
//     fn test_state_new_creates_player_and_mobs() {
//         let map = make_test_map();
//         let state = State::new(&map);

//         assert_eq!(state.player.unit.tile_x, 0);
//         assert_eq!(state.player.unit.tile_y, 0);
//         assert_eq!(state.player.unit.x_speed, 10.0);
//         assert_eq!(state.player.unit.y_speed, 10.0);

//         assert_eq!(state.mobs.len(), 2);

//         let mob_right = state.mobs.iter().find(|m| m.x_speed > 0.0).unwrap();
//         assert_eq!(mob_right.x_speed, 1.0);
//         assert_eq!(mob_right.y_speed, 0.0);
//         assert_eq!(mob_right.tile_x, 10);
//         assert_eq!(mob_right.tile_y, 0);

//         let mob_up = state.mobs.iter().find(|m| m.y_speed < 0.0).unwrap();
//         assert_eq!(mob_up.x_speed, 0.0);
//         assert_eq!(mob_up.y_speed, -0.5);
//         assert_eq!(mob_up.tile_x, 0);
//         assert_eq!(mob_up.tile_y, 10);
//     }

//     #[test]
//     fn test_state_with_no_mobs_other_than_player() {
//         let mut map = make_test_map();
//         map.mobs.retain(|_, mob| mob.is_player);
//         let state = State::new(&map);

//         assert_eq!(state.player.unit.tile_x, 0);
//         assert_eq!(state.player.unit.tile_y, 0);
//         assert!(state.mobs.is_empty());
//     }

//     #[test]
//     fn test_mob_with_unknown_or_none_behaviour_defaults_to_zero_speed() {
//         let mut mobs = std::collections::HashMap::new();

//         mobs.insert(
//             "player".to_string(),
//             Mob {
//                 name: "player".to_string(),
//                 x_start: 0,
//                 y_start: 0,
//                 asset: "knight".to_string(),
//                 is_player: true,
//                 behaviour: None,
//             },
//         );
//         mobs.insert(
//             "mob_none".to_string(),
//             Mob {
//                 name: "mob_none".to_string(),
//                 x_start: 5,
//                 y_start: 5,
//                 asset: "dummy".to_string(),
//                 is_player: false,
//                 behaviour: None,
//             },
//         );
//         mobs.insert(
//             "mob_unknown".to_string(),
//             Mob {
//                 name: "mob_unknown".to_string(),
//                 x_start: 10,
//                 y_start: 10,
//                 asset: "dummy".to_string(),
//                 is_player: false,
//                 behaviour: Some(Behaviour {
//                     behaviour_type: BehaviourType::Unknown,
//                     direction: Some("left".to_string()),
//                     speed: Some(2.0),
//                 }),
//             },
//         );

//         let map = GameMap {
//             name: "test_map".to_string(),
//             tile_size: 16,
//             size: [5, 5],
//             mobs,
//             objects: std::collections::HashMap::new(),
//             tiles: std::collections::HashMap::new(),
//             walk_map: vec![TileType::Empty; 25],
//         };

//         let state = State::new(&map);
//         assert_eq!(state.mobs.len(), 2);

//         let mob_none = state.mobs.iter().find(|m| m.tile_x == 5 && m.tile_y == 5).unwrap();
//         assert_eq!(mob_none.x_speed, 0.0);
//         assert_eq!(mob_none.y_speed, 0.0);

//         let mob_unknown = state.mobs.iter().find(|m| m.tile_x == 10 && m.tile_y == 10).unwrap();
//         assert_eq!(mob_unknown.x_speed, -2.0);
//         assert_eq!(mob_unknown.y_speed, 0.0);
//     }

//     #[test]
//     fn test_player_position_does_not_change_from_map() {
//         let map = make_test_map();
//         let state = State::new(&map);

//         let player_map = map.get_mob("player").unwrap();
//         assert_eq!(state.player.unit.tile_x, player_map.x_start as i32);
//         assert_eq!(state.player.unit.tile_y, player_map.y_start as i32);
//     }
// }
