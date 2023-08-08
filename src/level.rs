use macroquad::prelude::*;

use crate::fx;

pub struct Wall {
    pub points: Vec<Vec2>,
}

pub struct Level {
    pub walls: Vec<Wall>,
    pub start: Vec2,
    pub mesh: Mesh,
    material: Material,
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
    pub async fn load(path: &str) -> Result<Level, std::io::Error> {
        let material = load_material(
            ShaderSource {
                glsl_vertex: Some(
                    "#version 100
precision lowp float;
attribute vec3 position;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = position.xy;
}",
                ),
                glsl_fragment: Some(
                    "#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec2 f = fract(uv / 32.0);
    vec2 a = abs(f * 2.0 - vec2(1.0));
    float x = pow(max(a.x, a.y), 20.0);
    gl_FragColor = mix(
        vec4(0.11, 0.34, 0.22, 1.0),
        vec4(0.11, 0.4, 0.3, 1.0),
        x);
}
",
                ),
                metal_shader: None,
            },
            Default::default(),
        )
        .unwrap();

        let mut level = Level {
            walls: vec![],
            start: vec2(0.0, 0.0),
            mesh: Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            },
            material,
        };

        let string = macroquad::file::load_string(path).await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&string)?;

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
                }
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

    pub fn draw(&self) {
        gl_use_material(&self.material);
        draw_mesh(&self.mesh);
        gl_use_default_material();

        // for wall in self.walls.iter() {
        //     for (i, p) in wall.points.iter().enumerate() {
        //         let q = wall.points[(i + 1) % wall.points.len()];
        //         draw_line(p.x, p.y, q.x, q.y, 1.0, DARKBROWN);
        //     }
        // }
    }
}
