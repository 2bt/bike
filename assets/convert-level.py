#!/usr/bin/python
import json
import argparse
from pathlib import Path



def fix_points(points):
    s = 0
    for i, (px, py) in enumerate(points):
        qx, qy = points[(i + 1) % len(points)]
        s += (qx - px) * (qy + py)
    if s < 0: points = reversed(points)
    return points


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("--output", "-o")
    args = parser.parse_args()

    in_name = Path(args.input)
    out_name = args.output or in_name.with_suffix(".txt")

    out_file = open(out_name, "wt")

    j = json.load(open(in_name))


    for l in j["layers"]:
        if l["name"] == "walls":
            for o in l["objects"]:
                x = o["x"]
                y = o["y"]
                points = []
                for p in o["polygon"]:
                    px = x + p["x"]
                    py = y + p["y"]
                    points.append((px, py))
                s = ' '.join(f"({x} {y})" for x, y in fix_points(points))
                out_file.write(f"wall [{s}]\n")
        elif l["name"] == "objects":
            for o in l["objects"]:
                name = o["name"]
                x = o["x"]
                y = o["y"]
                out_file.write(f"{name} ({x} {y})\n")
