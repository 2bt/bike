use macroquad::prelude::*;
use std::f32::consts::PI;

type Vert = macroquad::models::Vertex;

pub fn vert(p: Vec2, color: Color) -> Vert {
    Vert {
        position: p.extend(0.0),
        uv: Vec2::ZERO,
        color: color.into(),
        normal: Vec4::ZERO,
    }
}

// triangle fan
// TODO: handle all concave polygons
pub fn draw_polygon(points: &[Vec2], color: Color) {
    let mut iter = points.iter().map(|p| vert(*p, color));
    let mut vertices = vec![
        iter.next().unwrap(),
        iter.next().unwrap(), //
    ];
    let mut indices = vec![];
    for (i, v) in iter.enumerate() {
        let i = i as u16;
        vertices.push(v);
        indices.extend_from_slice(&[0, i + 1, i + 2]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

pub fn draw_wheel(pos: Vec2, ang: f32, radius: f32, color: Color) {
    const N: u16 = 7;

    const W: f32 = 2.0;
    let radius = radius - W * 0.5;
    let mut vertices = vec![];
    let mut indices = vec![];

    for i in 0..=N {
        let f = i as f32 / N as f32 * 2.0 * PI;
        let n = Vec2::from_angle(ang + f);
        let f = (i as f32 - 0.5) / N as f32 * 2.0 * PI;
        let b1 = Vec2::from_angle(ang + f) * W;
        let f = (i as f32 + 0.5) / N as f32 * 2.0 * PI;
        let b2 = Vec2::from_angle(ang + f) * W;

        vertices.push(vert(pos + n * radius + b1, color));
        vertices.push(vert(pos + n * radius, color));
        vertices.push(vert(pos + n * radius + b2, color));

        if i > 0 {
            indices.extend_from_slice(&[
                i * 3 - 3,
                i * 3 - 2,
                i * 3 - 1,
                //
                i * 3 - 2,
                i * 3 - 1,
                i * 3,
                //
                i * 3 - 2,
                i * 3,
                i * 3 + 1,
            ])
        }
    }

    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

pub fn draw_limb(p: Vec2, q: Vec2, w: f32, v: f32, c: Color) {
    const N: u16 = 8;
    let w = w * 0.5;
    let v = v * 0.5;

    let pq = q - p;
    let wv = v - w;

    let alpha = pq.x.atan2(pq.y);
    let beta = wv.atan2((pq.length_squared() - wv * wv).sqrt());
    let a1 = PI - alpha + beta;
    let a2 = 2.0 * PI - alpha - beta;

    let mut vertices = vec![
        vert(p, c), //
        vert(q, c),
    ];
    let mut indices = vec![];

    for i in 0..=N {
        let m = i as f32 / N as f32;
        let a = a1 * (1.0 - m) + a2 * m;
        let pp = p + Vec2::from_angle(a) * w;
        draw_line(p.x, p.y, pp.x, pp.y, 1.0, c);
        vertices.push(vert(pp, c));
        if i > 0 {
            indices.extend_from_slice(&[
                0, //
                i + 1,
                i + 2,
            ]);
        }
    }

    let a1 = a1 + 2.0 * PI;
    for i in 0..=N {
        let m = i as f32 / N as f32;
        let a = a2 * (1.0 - m) + a1 * m;
        let qq = q + Vec2::from_angle(a) * v;
        draw_line(q.x, q.y, qq.x, qq.y, 1.0, c);
        vertices.push(vert(qq, c));
        if i > 0 {
            indices.extend_from_slice(&[
                1, //
                N + i + 2,
                N + i + 3,
            ]);
        }
    }

    indices.extend_from_slice(&[
        0,
        1,
        2, //
        1,
        2,
        N + N + 3,
        1,
        0,
        N + 3,
        0,
        N + 3,
        N + 2,
    ]);

    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

fn orientation(p: Vec2, q: Vec2, r: Vec2) -> bool {
    let val = (q - p).perp_dot(r - q);
    return val <= 0.0;
}
fn is_inside_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    orientation(p, a, b) && orientation(p, b, c) && orientation(p, c, a)
}
fn is_ear(polygon: &[Vec2], todo: &[usize], i: usize) -> bool {
    let n = todo.len();

    let ear_i = todo[i];
    let prev_i = todo[if i == 0 { n - 1 } else { i - 1 }];
    let next_i = todo[if i == n - 1 { 0 } else { i + 1 }];

    let ear_p = polygon[ear_i];
    let prev_p = polygon[prev_i];
    let next_p = polygon[next_i];

    if !orientation(prev_p, ear_p, next_p) {
        return false;
    }
    for j in todo.iter() {
        let j = *j;
        if (j == prev_i) || (j == ear_i) || (j == next_i) {
            continue;
        }
        let q = polygon[j];
        if is_inside_triangle(q, prev_p, ear_p, next_p) {
            return false;
        }
    }
    true
}

pub fn triangulate_polygon(polygon: &[Vec2]) -> Vec<u16> {
    if polygon.len() < 3 {
        return Vec::new();
    }

    let mut indices: Vec<u16> = Vec::new();
    let mut todo: Vec<usize> = (0..polygon.len()).collect();

    while todo.len() > 2 {
        let num_remaining = todo.len();
        for i in 0..num_remaining {
            if is_ear(polygon, &todo, i) {
                let n = todo.len();
                let ear_i = todo[i];
                let prev_i = todo[if i == 0 { n - 1 } else { i - 1 }];
                let next_i = todo[if i == n - 1 { 0 } else { i + 1 }];
                todo.remove(i);
                indices.push(prev_i as u16);
                indices.push(ear_i as u16);
                indices.push(next_i as u16);
                break;
            }
        }
    }
    indices
}
