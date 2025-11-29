import json

# ==========================================
# CONFIGURATION
# ==========================================

# ------------------------------------------
# LEGEND:
# ' ' or '-' : Floor
# '#'        : Wall (creates Concrete tile + Wall object)
# '.'        : Target (creates Target tile)
# '$'        : Box (creates Floor tile + Box mob)
# '*'        : Box on Target (creates Target tile + Box mob)
# '@'        : Player (creates Floor tile + Player mob)
# '+'        : Player on Target (creates Target tile + Player mob)
# ------------------------------------------

LEVEL_NAME = "level2"
OUTPUT_FILENAME = f"{LEVEL_NAME}.json"

LEVEL_LAYOUT = [
    "#  #",
    "#  #",
    ".$$ ",
    " $@.",
    "#. #"
]

# ==========================================
# GENERATOR LOGIC
# ==========================================

def generate_json(name, layout, filename):
    height = len(layout)
    width = max(len(row) for row in layout) if height > 0 else 0

    data = {
        "meta": {
            "name": name,
            "tile_size": 128,
            "size": [width, height]
        },
        "tiles": {},
        "objects": {},
        "mobs": {}
    }

    tile_counter = 1
    wall_counter = 1
    box_counter = 1
    
    player_found = False

    for y, row in enumerate(layout):
        # Pad row with spaces if it's shorter than the max width
        padded_row = row.ljust(width, ' ')
        
        for x, char in enumerate(padded_row):
            
            # 1. DETERMINE BASE TILE
            # ----------------------
            tile_key = f"tile_{tile_counter}"
            tile_asset = "floor"    # Default
            tile_type = "empty"     # Default

            # Logic for specific tiles
            if char == '#':
                tile_asset = "concrete" # Requirement: Under wall is concrete
            elif char in ['.', '*', '+']:
                tile_asset = "target"
                tile_type = "target"
            
            data["tiles"][tile_key] = {
                "x": x,
                "y": y,
                "asset": tile_asset,
                "tile_type": tile_type
            }
            tile_counter += 1

            # 2. DETERMINE STATIC OBJECTS (Walls)
            # ----------------------
            if char == '#':
                wall_key = f"wall_{wall_counter}"
                data["objects"][wall_key] = {
                    "x": x,
                    "y": y,
                    "asset": "wall_tile",
                    "collidable": True
                }
                wall_counter += 1

            # 3. DETERMINE MOBS (Player / Boxes)
            # ----------------------
            
            # Check for Player (@) or Player on Target (+)
            if char in ['@', '+']:
                if player_found:
                    print(f"WARNING: Multiple players found at {x},{y}. Overwriting previous.")
                
                data["mobs"]["player"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": "idle_se_0",
                    "is_player": True,
                    "behaviour": {
                        "type": "controlled"
                    }
                }
                player_found = True

            # Check for Box ($) or Box on Target (*)
            if char in ['$', '*']:
                box_key = f"box_{box_counter}"
                data["mobs"][box_key] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": "box",
                    "is_player": False
                }
                box_counter += 1

    # Write to file
    try:
        with open(filename, 'w') as f:
            json.dump(data, f, indent=4)
        print(f"Success! Generated '{filename}' with dimensions {width}x{height}.")
        print(f"Stats: {wall_counter-1} Walls, {box_counter-1} Boxes.")
    except IOError as e:
        print(f"Error writing file: {e}")

if __name__ == "__main__":
    generate_json(LEVEL_NAME, LEVEL_LAYOUT, OUTPUT_FILENAME)
