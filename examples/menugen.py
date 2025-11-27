import json

NOP = "NOP"

A = "A"
B = "B"
C = "C"
D = "D"
E = "E"
F = "F"
G = "G"
H = "H"
I = "I"
J = "J"
K = "K"
L = "L"
M = "M"
N = "N"
O = "O"
P = "P"
Q = "Q"
R = "R"
S = "S"
T = "T"
U = "U"
V = "V"
W = "W"
X = "X"
Y = "Y"
Z = "Z"

NUM0 = "0"
NUM1 = "1"
NUM2 = "2"
NUM3 = "3"
NUM4 = "4"
NUM5 = "5"
NUM6 = "6"
NUM7 = "7"
NUM8 = "8"
NUM9 = "9"

PLAYER = "#"
# BOX?

map_menu = [
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [NOP, NOP, C, O, N, G, R, A, T, U, L, A, T, I, O, N, S, NOP, NOP],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [NOP, NOP, C, H, O, O, S, E, E, NOP, L, E, V, E, L, NOP, NOP, NOP],
    [
        NOP,
        PLAYER,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NUM1,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NUM2,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NUM3,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NUM4,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
    [
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
        NOP,
    ],
]
size = len(map_menu)


def generate_data(name, map, size):
    data = {
        "meta": {"name": name, "tile_size": 256, "size": [size, size]},
        "tiles": {},
        "objects": {},
        "mobs": {},
    }

    tile_count = 1
    letter_count = 1
    for y in range(size):
        for x in range(size):
            cell = map[y][x]

            data["tiles"][f"tile_{tile_count}"] = {"x": x, "y": y, "asset": "floor"}
            tile_count += 1

            if cell == NOP:
                continue
            elif cell == PLAYER:
                data["mobs"]["player"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": cell,
                    "is_player": True,
                    "behaviour": {"type": "controlled"},
                }
            else:
                data["mobs"][f"letter_{letter_count}"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": cell,
                    "is_player": False,
                }
                letter_count += 1

    return data


def save_data_to_file(name, map, size):
    data = generate_data(name, map, size)

    with open(f"{name}.json", "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

    return data


if __name__ == "__main__":
    save_data_to_file("menu", map_menu, 16)
