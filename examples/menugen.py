import json

_N = "NOP"
_P = "running_se_0"
_B = "box"

A_, B_, C_, D_, E_, F_, G_, H_, I_, J_, K_, L_, M_ = "ABCDEFGHIJKLM"
N_, O_, P_, Q_, R_, S_, T_, U_, V_, W_, X_, Y_, Z_ = "NOPQRSTUVWXYZ"

N0, N1, N2, N3, N4, N5, N6, N7, N8, N9 = "0123456789"

map_menu = [
    [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N],
    [_N, _N, C_, H_, O_, O_, S_, E_, _N, _N],
    [_N, _N, L_, E_, V_, E_, L_, _N, _N, _N],
    [_N, _P, _N, _N, _N, _N, _N, _N, _N, _N],
    [_N, _N, _N, _B, N1, _N, _N, _N, _N, _N],
    [_N, _N, _N, _B, N2, _N, _N, _N, _N, _N],
    [_N, _N, _N, _B, N3, _N, _N, _N, _N, _N],
    [_N, _N, _N, _B, N4, _N, _N, _N, _N, _N],
    [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N],
    [_N, _N, _N, _N, _N, _N, _N, _N, _N, _N],
]


def generate_data(name, map, size):
    data = {
        "meta": {"name": name, "tile_size": 256, "size": [size, size]},
        "tiles": {},
        "objects": {},
        "mobs": {},
    }

    # tile_count = 1
    letter_count = 1
    box_count = 1
    for y in range(size):
        for x in range(size):
            cell = map[y][x]

            # tile_id = f"tile_{tile_count}"
            # data["tiles"][tile_id] = {"x": x, "y": y, "asset": "floor"}
            # tile_count += 1

            if cell == _N:
                continue
            elif cell == _P:
                data["mobs"]["player"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": cell,
                    "is_player": True,
                    "behaviour": {"type": "controlled"},
                }
            elif cell == _B:
                box_id = f"box_{box_count}"
                data["mobs"][box_id] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": cell,
                    "is_player": False,
                }
                box_count += 1
            else:
                letter_id = f"letter_{letter_count}"
                data["tiles"][letter_id] = {
                    "x": x,
                    "y": y,
                    "asset": cell,
                }
                letter_count += 1

    return data


def save_data_to_file(name, map, size):
    data = generate_data(name, map, size)

    with open(f"{name}.json", "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

    return data


if __name__ == "__main__":
    size = len(map_menu)
    save_data_to_file("menu", map_menu, size)
