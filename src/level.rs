use macroquad::prelude::*;

use crate::{fx, materials};

const STAR_R: f32 = 10.0;


pub struct CollisionInfo {
    pub normal: Vec2,
    pub dist: f32,
}

pub struct Wall {
    pub points: Vec<Vec2>,
}

pub struct Star {
    pub alive: bool,
    pub pos: Vec2,
}

pub struct Level {
    pub walls: Vec<Wall>,
    pub start: Vec2,
    pub stars: Vec<Star>,
    pub stars_left: usize,
    pub mesh: Mesh,
}

fn circle_line_collision(m: Vec2, r: f32, p: Vec2, q: Vec2) -> Option<CollisionInfo> {
    let pq = q - p;
    let pm = m - p;
    let e = pq.dot(pm);
    if e < 0.0 {
        let dist = pm.length();
        if dist < r {
            return Some(CollisionInfo {
                normal: pm.normalize(),
                dist: r - dist,
            });
        }
    } else if e < pq.length_squared() {
        let mut norm = pq.perp().normalize();
        let mut dist = norm.dot(pm);
        if dist < 0.0 {
            dist = -dist;
            norm = -norm;
        }
        if dist < r {
            return Some(CollisionInfo {
                normal: norm,
                dist: r - dist,
            });
        }
    } else {
        let qm = m - q;
        let dist = qm.length();
        if dist < r {
            return Some(CollisionInfo {
                normal: qm.normalize(),
                dist: r - dist,
            });
        }
    }
    None
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
            stars_left: 0,
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
                                level.stars.push(Star { alive: true, pos });
                                level.stars_left += 1;
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

    pub fn circle_collision(&self, pos: Vec2, r: f32) -> Option<CollisionInfo> {
        let mut colli: Option<CollisionInfo> = None;
        for wall in self.walls.iter() {
            for (i, p) in wall.points.iter().enumerate() {
                let q = wall.points[(i + 1) % wall.points.len()];
                if let Some(ci) = circle_line_collision(pos, r, *p, q) {
                    colli = Some(match colli {
                        Some(cc) if cc.dist > ci.dist => cc,
                        _ => ci,
                    })
                }
            }
        }
        colli
    }

    pub fn pickup_stars(&mut self, pos: Vec2, r: f32) {
        let l = r + STAR_R;
        let l = l * l;
        for star in self.stars.iter_mut() {
            if !star.alive { continue; }
            if (pos - star.pos).length_squared() <= l {
                star.alive = false;
                self.stars_left -= 1;
            }
        }
    }
    pub fn reset_stars(&mut self) {
        for star in self.stars.iter_mut() {
            star.alive = true;
        }
        self.stars_left = self.stars.len();
    }


    pub fn draw(&self, materials: &materials::Materials) {
        gl_use_material(&materials.wall_material);
        draw_mesh(&self.mesh);
        gl_use_default_material();

        for star in self.stars.iter() {
            if !star.alive { continue; }
            draw_circle(star.pos.x, star.pos.y, 9.0, Color::from_hex(0xffff00));
        }
    }
}
