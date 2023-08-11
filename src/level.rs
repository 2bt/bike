use macroquad::prelude::*;

use crate::{fx, materials};

pub struct Wall {
    pub points: Vec<Vec2>,
}

pub struct Level {
    pub walls: Vec<Wall>,
    pub start: Vec2,
    pub stars: Vec<Vec2>,
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

fn vec_from_json(json: &serde_json::Value) -> Vec2 {
    Vec2 {
        x: json["x"].as_f64().unwrap() as f32,
        y: json["y"].as_f64().unwrap() as f32,
    }
}

impl Level {
    pub async fn load(path: &str) -> Result<Level, std::io::Error> {
        let mut level = Level {
            walls: vec![],
            start: vec2(0.0, 0.0),
            stars: vec![],
            mesh: Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            },
        };

        let string = macroquad::file::load_string(path).await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&string)?;

        for layer in json["layers"].as_array().unwrap() {
            match layer["name"].as_str().unwrap() {
                "walls" => {
                    for o in layer["objects"].as_array().unwrap() {
                        let mut wall = Wall { points: vec![] };
                        let pos = vec_from_json(o);
                        for p in o["polygon"].as_array().unwrap() {
                            wall.points.push(pos + vec_from_json(p));
                        }
                        fix_points(&mut wall.points);
                        level.walls.push(wall);
                    }
                }
                "objects" => {
                    for o in layer["objects"].as_array().unwrap() {
                        let name = o["name"].as_str().unwrap();
                        let pos = vec_from_json(o);
                        match name {
                            "start" => {
                                level.start = pos;
                            }
                            "star" => {
                                level.stars.push(pos);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
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

    pub fn draw(&self, materials: &materials::Materials) {
        gl_use_material(&materials.wall_material);
        draw_mesh(&self.mesh);
        gl_use_default_material();

        for star in self.stars.iter() {
            draw_circle(star.x, star.y, 9.0, Color::from_hex(0xffff00));
        }
    }
}
