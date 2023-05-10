use macroquad::prelude::*;
use std::f32::consts::PI;

type Vert = macroquad::models::Vertex;

fn vert(p: Vec2, color: Color) -> Vert {
    Vert {
        position: p.extend(0.0),
        uv: vec2(0.0, 0.0),
        color,
    }
}

// pub fn draw_wheel(p: Vec2, ) {

// }

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

    let mut mesh = Mesh {
        vertices: vec![
            vert(p, c), //
            vert(q, c),
        ],
        indices: vec![],
        texture: None,
    };

    for i in 0..=N {
        let m = i as f32 / N as f32;
        let a = a1 * (1.0 - m) + a2 * m;
        let pp = p + Vec2::from_angle(a) * w;
        draw_line(p.x, p.y, pp.x, pp.y, 1.0, c);
        mesh.vertices.push(vert(pp, c));
        if i > 0 {
            mesh.indices.extend_from_slice(&[
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
        mesh.vertices.push(vert(qq, c));
        if i > 0 {
            mesh.indices.extend_from_slice(&[
                1, //
                N + i + 2,
                N + i + 3,
            ]);
        }
    }

    mesh.indices.extend_from_slice(&[
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

    draw_mesh(&mesh);
}
