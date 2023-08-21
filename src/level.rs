use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::fx;
use crate::materials;

const STAR_R: f32 = 10.0;

pub enum PolygonType {
    Wall,
    Lava,
}

pub struct CollisionInfo {
    pub normal: Vec2,
    pub dist: f32,
    pub tpe: PolygonType,
}

struct Polygon {
    tpe: PolygonType,
    points: Vec<Vec2>,
}

pub struct Star {
    alive: bool,
    pos: Vec2,
}

pub struct Level {
    pub start: Vec2,
    pub stars: Vec<Star>,
    pub stars_left: usize,
    polygons: Vec<Polygon>,
    time: f32,
    wall_mesh: Mesh,
    lava_mesh: Mesh,
}

impl Default for Level {
    fn default() -> Self {
        Level {
            polygons: vec![],
            start: vec2(0.0, 0.0),
            stars: vec![],
            stars_left: 0,
            time: 0.0,
            wall_mesh: Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            },
            lava_mesh: Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            },
        }
    }
}

fn circle_line_collision(m: Vec2, r: f32, p: Vec2, q: Vec2) -> Option<CollisionInfo> {
    let pq = q - p;
    let pm = m - p;
    let e = pq.dot(pm);
    if e < 0.0 {
        let dist = pm.length();
        if dist < r {
            return Some(CollisionInfo {
                tpe: PolygonType::Wall,
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
                tpe: PolygonType::Wall,
                normal: norm,
                dist: r - dist,
            });
        }
    } else {
        let qm = m - q;
        let dist = qm.length();
        if dist < r {
            return Some(CollisionInfo {
                tpe: PolygonType::Wall,
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
        let mut level = Level::default();
        let string = macroquad::file::load_string(path).await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&string)?;

        for layer in json["layers"].as_array().unwrap() {
            let name = layer["name"].as_str().unwrap();
            match name {
                "walls" | "lava" => {
                    for o in layer["objects"].as_array().unwrap() {
                        let mut poly = Polygon {
                            tpe: match name {
                                "walls" => PolygonType::Wall,
                                _ => PolygonType::Lava,
                            },
                            points: vec![],
                        };
                        let pos = vec_from_json(o);
                        for p in o["polygon"].as_array().unwrap() {
                            poly.points.push(pos + vec_from_json(p));
                        }
                        fix_points(&mut poly.points);
                        level.polygons.push(poly);
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
        let color = Color::new(1.0, 1.0, 1.0, 1.0);
        for poly in level.polygons.iter() {
            let mesh = match poly.tpe {
                PolygonType::Wall => &mut level.wall_mesh,
                PolygonType::Lava => &mut level.lava_mesh,
            };

            let n = mesh.vertices.len();
            mesh.vertices
                .extend(poly.points.iter().map(|p| fx::vert(*p, color)));
            let indices = fx::triangulate_polygon(&poly.points);
            mesh.indices.extend(indices.iter().map(|i| *i + n as u16));
        }

        Ok(level)
    }

    pub fn circle_collision(&self, pos: Vec2, r: f32) -> Option<CollisionInfo> {
        let mut colli: Option<CollisionInfo> = None;
        for poly in self.polygons.iter() {
            for (i, p) in poly.points.iter().enumerate() {
                let q = poly.points[(i + 1) % poly.points.len()];
                if let Some(ci) = circle_line_collision(pos, r, *p, q) {
                    if let PolygonType::Lava = poly.tpe {
                        return Some(CollisionInfo {
                            tpe: PolygonType::Lava,
                            ..ci
                        });
                    }
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
            if !star.alive {
                continue;
            }
            if (pos - star.pos).length_squared() <= l {
                star.alive = false;
                self.stars_left -= 1;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn draw(&self, materials: &materials::Materials) {

        gl_use_material(&materials.wall_material);
        draw_mesh(&self.wall_mesh);

        gl_use_material(&materials.lava_material);
        draw_mesh(&self.lava_mesh);

        gl_use_default_material();

        let c = Color::new(0.8, 0.8, 0.3, 1.0);
        for star in self.stars.iter() {
            if !star.alive {
                continue;
            }
            let mut points = [Vec2::ZERO; 12];
            points[0] = star.pos;
            for i in 1..points.len() {
                let r = if i % 2 == 0 { STAR_R } else { STAR_R * 0.5 };
                let ang = (i as f32) * 0.2 * PI + (self.time * 3.0).sin() * 0.8;
                points[i] = star.pos - Vec2::from_angle(ang) * r;
            }
            fx::draw_polygon(&points, c);
        }
    }
}
