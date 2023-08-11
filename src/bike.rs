use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::fx;
use crate::level;

const GRAVITY: f32 = 100.0;
const FRAME_MASS: f32 = 20.0;
const FRAME_INERTIA: f32 = 5000.0;
const WHEEL_MASS: f32 = 1.0;
const WHEEL_INERTIA: f32 = 50.0;
const WHEEL_R: f32 = 8.0;
const WHEEL_X: f32 = 17.0;
const WHEEL_Y: f32 = 12.0;
const SUSPENSION: f32 = 700.0;
const SUSPENSION_FRICTION: f32 = 40.0;
// const BREAK: f32 = 60000.0;
// const BREAK_FRICTION: f32 = 40000.0;
const BREAK: f32 = 60000.0 * 0.5;
const BREAK_FRICTION: f32 = 40000.0 * 0.5;
const MAX_SPEED: f32 = 50.0;
const GAS: f32 = 17000.0;

struct CollisionInfo {
    normal: Vec2,
    dist: f32,
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

impl level::Level {
    fn wheel_collision(&self, wheel: &Body) -> Option<CollisionInfo> {
        let mut colli: Option<CollisionInfo> = None;
        for wall in self.walls.iter() {
            for (i, p) in wall.points.iter().enumerate() {
                let q = wall.points[(i + 1) % wall.points.len()];
                if let Some(ci) = circle_line_collision(wheel.pos, WHEEL_R, *p, q) {
                    colli = Some(match colli {
                        Some(cc) if cc.dist > ci.dist => cc,
                        _ => ci,
                    })
                }
            }
        }
        colli
    }
}

pub struct Body {
    pub pos: Vec2,
    ang: f32,
    vel: Vec2,
    ang_vel: f32,
    force: Vec2,
    torque: f32,
}
impl Default for Body {
    fn default() -> Self {
        return Body {
            pos: vec2(0.0, 0.0),
            ang: 0.0,
            vel: vec2(0.0, 0.0),
            ang_vel: 0.0,
            force: vec2(0.0, 0.0),
            torque: 0.0,
        };
    }
}

fn update_frame(frame: &mut Body, dt: f32) {
    frame.ang_vel += frame.torque / FRAME_INERTIA * dt;
    frame.vel += frame.force / FRAME_MASS * dt;
    frame.ang += frame.ang_vel * dt;
    frame.pos += frame.vel * dt;
}

fn update_wheel(wheel: &mut Body, dt: f32, ci: Option<CollisionInfo>) {
    if let Some(ci) = ci {
        wheel.pos += ci.normal * ci.dist;

        wheel.ang_vel = ci.normal.perp_dot(wheel.vel) / WHEEL_R;
        wheel.torque += ci.normal.perp_dot(wheel.force) * WHEEL_R;

        wheel.ang_vel += wheel.torque / WHEEL_INERTIA * dt;
        wheel.vel = ci.normal.perp() * wheel.ang_vel * WHEEL_R;
    } else {
        wheel.ang_vel += wheel.torque / WHEEL_INERTIA * dt;
        wheel.vel += wheel.force / WHEEL_MASS * dt;
    }
    wheel.ang += wheel.ang_vel * dt;

    wheel.pos += wheel.vel * dt;
}

pub enum Direction {
    Right,
    Left,
}

pub struct Bike {
    pub frame: Body,
    wheels: [Body; 2],
    breaking: bool,
    break_angles: [f32; 2],
    dir: Direction,
    dir_lerp: f32,
    dir_toggling: bool,
}

impl Bike {
    pub fn new(pos: Vec2) -> Bike {
        Bike {
            frame: Body {
                pos: pos,
                ..Default::default()
            },
            wheels: [
                Body {
                    pos: pos + vec2(-WHEEL_X, WHEEL_Y),
                    ..Default::default()
                },
                Body {
                    pos: pos + vec2(WHEEL_X, WHEEL_Y),
                    ..Default::default()
                },
            ],
            breaking: false,
            break_angles: [0.0; 2],
            dir: Direction::Right,
            dir_lerp: 1.0,
            dir_toggling: false,
        }
    }

    pub fn update(&mut self, dt: f32, level: &level::Level) {
        // toggle dir
        let dir_toggling = is_key_down(KeyCode::Space);
        if dir_toggling && !self.dir_toggling {
            self.dir = match self.dir {
                Direction::Right => Direction::Left,
                Direction::Left => Direction::Right,
            };
        }
        self.dir_toggling = dir_toggling;
        self.dir_lerp = match self.dir {
            Direction::Right => (1.0_f32).min(self.dir_lerp + dt * 20.0),
            Direction::Left => (-1.0_f32).max(self.dir_lerp - dt * 20.0),
        };

        // reset forces
        self.frame.torque = 0.0;
        self.frame.force = vec2(0.0, GRAVITY * FRAME_MASS);
        self.wheels[0].torque = 0.0;
        self.wheels[0].force = vec2(0.0, GRAVITY * WHEEL_MASS);
        self.wheels[1].torque = 0.0;
        self.wheels[1].force = vec2(0.0, GRAVITY * WHEEL_MASS);

        // move around
        if is_key_down(KeyCode::D) {
            self.frame.force.x += 5000.0;
        }
        if is_key_down(KeyCode::A) {
            self.frame.force.x -= 5000.0;
        }
        if is_key_down(KeyCode::S) {
            self.frame.force.y += 5000.0;
        }
        if is_key_down(KeyCode::W) {
            self.frame.force.y -= 5000.0;
        }

        // break
        let breaking = is_key_down(KeyCode::Down);
        if breaking && !self.breaking {
            for (i, wheel) in self.wheels.iter().enumerate() {
                self.break_angles[i] = wheel.ang - self.frame.ang;
            }
        }
        if breaking {
            for (i, wheel) in self.wheels.iter_mut().enumerate() {
                let da = wheel.ang - self.frame.ang - self.break_angles[i];
                let dv = wheel.ang_vel - self.frame.ang_vel;
                let torque = da * BREAK + dv * BREAK_FRICTION;
                wheel.torque -= torque;
                self.frame.torque += torque;
            }
        } else {
            // wrap angles
            self.frame.ang %= 2.0 * PI;
            self.wheels[0].ang %= 2.0 * PI;
            self.wheels[1].ang %= 2.0 * PI;
        }
        self.breaking = breaking;

        // gas
        let (wheel, sign) = match self.dir {
            Direction::Right => (&mut self.wheels[0], 1.0),
            Direction::Left => (&mut self.wheels[1], -1.0),
        };
        if is_key_down(KeyCode::Up) && wheel.ang_vel < MAX_SPEED {
            wheel.torque += GAS * sign;
            self.frame.torque -= GAS * sign;
            // XXX: is this correct?
            // let arm = wheel.pos - frame.pos;
            // frame.force += torque * arm.perp() / arm.length_squared();
        }

        // suspension
        let rot = Vec2::from_angle(self.frame.ang);
        for (i, wheel) in self.wheels.iter_mut().enumerate() {
            let arm = match i {
                0 => vec2(-WHEEL_X, WHEEL_Y),
                _ => vec2(WHEEL_X, WHEEL_Y),
            };
            let arm = arm.rotate(rot);
            let force = (self.frame.pos + arm - wheel.pos) * SUSPENSION;

            let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            self.frame.force -= force;
            self.frame.torque += force.perp_dot(arm);

            // friction
            // XXX: is this arm right?
            let arm = wheel.pos - self.frame.pos;

            let dv = self.frame.vel + self.frame.ang_vel * arm.perp() - wheel.vel;
            let force = dv * SUSPENSION_FRICTION;

            let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            self.frame.force -= force;
            self.frame.torque += force.perp_dot(arm);
        }

        update_frame(&mut self.frame, dt);
        for w in self.wheels.iter_mut() {
            let ci = level.wheel_collision(w);
            update_wheel(w, dt, ci);
        }
    }

    pub fn draw(&self) {
        let trans = Affine2::from_scale_angle_translation(
            vec2(self.dir_lerp, 1.0), //
            self.frame.ang,
            self.frame.pos,
        );
        let t = |x: i32, y: i32| trans.transform_point2(vec2(x as f32, y as f32));

        // wheels
        let c = Color::from_rgba(130, 130, 130, 255);
        for w in self.wheels.iter() {
            fx::draw_wheel(w.pos, w.ang, WHEEL_R, c);
        }

        // springs
        let c = Color::from_rgba(140, 80, 70, 255);
        let lerp = 0.5 + (self.dir_lerp * 0.5 * PI).sin() * 0.5;
        let w0 = self.wheels[1].pos.lerp(self.wheels[0].pos, lerp);
        let w1 = self.wheels[0].pos.lerp(self.wheels[1].pos, lerp);
        fx::draw_limb(t(0, 9), w0, 3.0, 3.0, c);
        fx::draw_limb(t(12, -1), w1, 3.0, 3.0, c);

        // frame
        fx::draw_polygon(
            &[
                t(2, -3),
                t(9, -9),
                t(14, -4),
                t(1, 11),
                t(-1, 11),
                t(-11, 0),
                t(-17, -2),
                t(-17, -7),
                t(-10, -7),
            ],
            Color::from_rgba(70, 60, 50, 255),
        );

        // rider
        let c = Color::from_rgba(130, 130, 130, 255);
        let limb = |x1: i32, y1: i32, x2: i32, y2: i32, w: f32, v: f32, c: Color| {
            fx::draw_limb(t(x1, y1), t(x2, y2), w, v, c);
        };
        // head
        let q = t(0, -21);
        draw_poly(q.x, q.y, 16, 4.5, self.frame.ang * (180.0 / PI), c);
        // body
        limb(-2, -16, -10, -9, 6.0, 6.0, c);
        // leg
        limb(-10, -9, -2, -3, 6.0, 4.0, c);
        limb(-2, -3, -1, 6, 4.0, 3.0, c);
        limb(-1, 6, 2, 6, 3.0, 2.0, c);
        // arm
        limb(-1, -15, 2, -8, 4.0, 3.0, c);
        limb(2, -8, 10, -7, 3.0, 2.5, c);
    }
}
