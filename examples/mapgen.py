import json

F = 1  # Floor
W = 2  # Wall
B = 3  # Box
T = 4  # Target
P = 5  # Player
G = 6  # Goal = Box + Target


def create_empty_map(size):
    map = [[F for _ in range(size)] for _ in range(size)]
    return map


def create_room_map(size, border):
    assert 0 <= border and border < size

    map = create_empty_map(size)

    for y in range(border - 1, size - border):
        map[y][border - 1] = W
        map[y][size - border - 1] = W

    for x in range(border - 1, size - border):
        map[border - 1][x] = W
        map[size - border - 1][x] = W

    return map


def generate_data(name, map, size):
    data = {
        "meta": {"name": name, "tile_size": 256, "size": [size, size]},
        "tiles": {},
        "objects": {},
        "mobs": {},
    }

    tile_count = 1
    wall_count = 1
    box_count = 1
    for y in range(size):
        for x in range(size):
            cell = map[y][x]

            tile_id = f"tile_{tile_count}"

            if cell == T or cell == G:
                data["tiles"][tile_id] = {
                    "x": x,
                    "y": y,
                    "asset": "target",
                    "tile_type": "target",
                }
            else:
                data["tiles"][tile_id] = {
                    "x": x,
                    "y": y,
                    "asset": "floor",
                    "tile_type": "empty",
                }

            tile_count += 1

            if cell == W:
                wall_id = f"wall_{wall_count}"
                data["objects"][wall_id] = {
                    "x": x,
                    "y": y,
                    "asset": "wall_tile",
                    "collidable": True,
                }
                wall_count += 1
            elif cell == B or cell == G:
                box_id = f"box_{box_count}"
                data["mobs"][box_id] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": "box",
                    "is_player": False,
                }
                box_count += 1
            elif cell == P:
                data["mobs"]["player"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": "running_se_0",
                    "is_player": True,
                    "behaviour": {"type": "controlled"},
                }

    return data


def save_data_to_file(name, map, size):
    data = generate_data(name, map, size)

    with open(f"{name}.json", "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

    return data


if __name__ == "__main__":
    # Level 1
    size1 = 5
    map1 = create_empty_map(size1)
    map1[0][0] = T
    map1[0][1] = B
    map1[size1 // 2][size1 // 2] = P
    save_data_to_file("level1", map1, size1)

    # Level 2
    size2 = 10
    border2 = 2
    map2 = create_room_map(size2, border2 + 1)
    map2[border2 + 1][border2 - 1] = F
    map2[border2 + 2][border2 + 1] = T
    map2[border2][border2 + 1] = T
    map2[4][1] = B
    map2[border2 - 1][6] = B
    map2[size2 // 2][size2 // 2] = P
    save_data_to_file("level2", map2, size2)

    # Level 3
    map3 = [
        [F, F, F, F, F, F, F, F, F, F],
        [F, F, F, W, W, W, W, F, F, F],
        [F, F, W, W, F, F, W, F, F, F],
        [F, F, W, P, B, F, W, F, F, F],
        [F, F, W, W, B, F, W, W, F, F],
        [F, F, W, W, F, B, F, W, F, F],
        [F, F, W, T, B, F, F, W, F, F],
        [F, F, W, T, T, G, T, W, F, F],
        [F, F, W, W, W, W, W, W, F, F],
        [F, F, F, F, F, F, F, F, F, F],
    ]
    save_data_to_file("level3", map3, 10)

    # Level 4
    map4 = [
        [F, F, F, F, F, F, F, F, F, F, F],
        [F, F, W, W, W, W, W, W, W, F, F],
        [W, W, W, F, F, F, F, F, W, F, F],
        [F, F, F, T, W, W, W, F, W, F, F],
        [F, W, F, W, F, F, F, F, W, W, F],
        [F, W, F, B, F, B, W, T, F, W, F],
        [F, W, F, F, G, F, F, W, F, W, F],
        [F, T, W, B, F, B, F, W, F, W, F],
        [W, F, F, F, F, W, F, W, F, W, W],
        [W, F, W, W, W, T, F, F, F, F, P],
        [W, F, F, F, F, F, W, W, F, F, F],
    ]
    save_data_to_file("level4", map4, 11)
