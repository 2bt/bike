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

pub enum Line {
    Wall(Wall),
    Start(Vec2),
}

peg::parser! {
    grammar parser() for str {
        rule _() = quiet!{[' '|'\t']*}
        rule number() -> f32 = _ n:$("-"?['0'..='9']+) { n.parse().unwrap() }
        rule point() -> Vec2 = _ "(" x:number() y:number() _ ")" { vec2(x, y) }
        rule points() -> Vec<Vec2> = "[" p:point()* _ "]" { p }
        pub rule line() -> Line
            = "wall" _ points:points() { Line::Wall(Wall { points }) }
            / "start" _ p:point() { Line::Start(p) }
    }
}

impl Level {
    pub fn load(path: &str) -> Result<Level, ()> {
        let mut level = Level {
            walls: vec![],
            start: vec2(0.0, 0.0),
            mesh: Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            },
        };

        // parse file
        use std::io::BufRead;
        let file = std::fs::File::open(path).map_err(|_| ())?;
        for line in std::io::BufReader::new(file).lines() {
            let line = line.map_err(|_| ())?;
            match parser::line(line.as_str()) {
                Ok(Line::Wall(w)) => level.walls.push(w),
                Ok(Line::Start(p)) => level.start = p,
                _ => return Err(()),
            }
        }

        // generate mesh
        for wall in level.walls.iter() {
            let n = level.mesh.vertices.len();
            level
                .mesh
                .vertices
                .extend(wall.points.iter().map(|p| fx::vert(*p, DARKGREEN)));
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
