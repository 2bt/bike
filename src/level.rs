use macroquad::prelude::*;

use crate::fx;

pub struct Wall {
    pub points: Vec<Vec2>,
}

pub struct Level {
    pub walls: Vec<Wall>,
    pub start: Vec2,
    pub mesh: Mesh,
}

fn fix_points(points: &mut Vec<Vec2>) {
    let mut s = 0.0;
    for i in 0..points.len() {
        let p = &points[i];
        let q = &points[(i + 1) % points.len()];
        s += (q.x - p.x) * (q.y + p.y);
    }
    if s < 0.0 {
        points.reverse();
    }
}

impl Level {
    pub fn load(path: &str) -> Result<Level, std::io::Error> {
        let mut level = Level {
            walls: vec![],
            start: vec2(0.0, 0.0),
            mesh: Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            },
        };

        let in_file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(in_file);
        let json: serde_json::Value = serde_json::from_reader(reader)?;

        for layer in json["layers"].as_array().unwrap() {
            match layer["name"].as_str().unwrap() {
                "walls" => {
                    for o in layer["objects"].as_array().unwrap() {
                        let mut wall = Wall { points: vec![] };
                        let pos = vec2(
                            o["x"].as_f64().unwrap() as f32,
                            o["y"].as_f64().unwrap() as f32,
                        );
                        for p in o["polygon"].as_array().unwrap() {
                            let p = vec2(
                                p["x"].as_f64().unwrap() as f32,
                                p["y"].as_f64().unwrap() as f32,
                            );
                            wall.points.push(pos + p);
                        }
                        fix_points(&mut wall.points);
                        level.walls.push(wall);
                    }
                },
                "objects" => {
                    for o in layer["objects"].as_array().unwrap() {
                        let name = o["name"].as_str().unwrap();
                        let pos = vec2(
                            o["x"].as_f64().unwrap() as f32,
                            o["y"].as_f64().unwrap() as f32,
                        );
                        match name {
                            "start" => {
                                level.start = pos;
                            },
                            _ => {},
                        }
                    }
                },
                _ => {},
            }
        }

        // generate mesh
        let color = Color::from_rgba(30, 100, 50, 255);
        for wall in level.walls.iter() {
            let n = level.mesh.vertices.len();
            level
                .mesh
                .vertices
                .extend(wall.points.iter().map(|p| fx::vert(*p, color)));
            let indices = fx::triangulate_polygon(&wall.points);
            level
                .mesh
                .indices
                .extend(indices.iter().map(|i| *i + n as u16));
        }

        Ok(level)
    }

    pub fn draw(&self) {
        draw_mesh(&self.mesh);

        // for wall in self.walls.iter() {
        //     for (i, p) in wall.points.iter().enumerate() {
        //         let q = wall.points[(i + 1) % wall.points.len()];
        //         draw_line(p.x, p.y, q.x, q.y, 1.0, DARKBROWN);
        //     }
        // }
    }
}
