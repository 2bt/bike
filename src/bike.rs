use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::fx;
use crate::level;
use crate::level::CollisionInfo;

const GRAVITY: f32 = 100.0;
const FRAME_MASS: f32 = 20.0;
const FRAME_INERTIA: f32 = 5000.0;
const WHEEL_MASS: f32 = 1.0;
const WHEEL_INERTIA: f32 = 50.0;
const WHEEL_R: f32 = 8.0;
const WHEEL_X: f32 = 17.0;
const WHEEL_Y: f32 = 12.0;
const SUSPENSION: f32 = 1200.0;
const SUSPENSION_FRICTION: f32 = 70.0;
// const BREAK: f32 = 60000.0 * 0.5;
// const BREAK_FRICTION: f32 = 40000.0 * 0.5;
const BREAK: f32 = 0.0; // disable this for now, it feels weird
const BREAK_FRICTION: f32 = 7000.0;
const MAX_SPEED: f32 = 50.0;
const GAS: f32 = 18000.0;

const JUMP_STRENGTH: f32 = 8.0;
const JUMP_DURATION: f32 = 0.1;
const JUMP_PAUSE: f32 = 0.5;

#[derive(PartialEq, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    Right,
    Left,
}

#[derive(PartialEq)]
pub enum WheelInput {
    None,
    Break,
    Accelerate,
}
pub struct Input {
    pub toggle_dir: bool,
    pub wheel: WheelInput,
    pub jump: Option<Direction>,
}

#[derive(Default)]
pub struct Body {
    pub pos: Vec2,
    ang: f32,
    vel: Vec2,
    ang_vel: f32,
    force: Vec2,
    torque: f32,
}

pub struct Jump {
    dir: Direction,
    time: f32,
    ang_vel: f32,
}

#[derive(Default)]
pub struct Bike {
    pub alive: bool,
    pub frame: Body,
    wheels: [Body; 2],
    breaking: bool,
    break_angles: [f32; 2],
    dir: Direction,
    dir_lerp: f32,
    prev_toggle_dir: bool,
    jump: Option<Jump>,
}

fn update_frame(frame: &mut Body, dt: f32) {
    frame.ang_vel += frame.torque / FRAME_INERTIA * dt;
    frame.vel += frame.force / FRAME_MASS * dt;
    frame.ang += frame.ang_vel * dt;
    frame.pos += frame.vel * dt;
    // reset forces
    frame.torque = 0.0;
    frame.force = vec2(0.0, GRAVITY * FRAME_MASS);
}

fn update_wheel(wheel: &mut Body, dt: f32, ci: Option<CollisionInfo>) {
    let mut b = false;
    if let Some(ci) = ci {
        wheel.pos += ci.normal * ci.dist;

        if ci.normal.dot(wheel.vel) < 0.0 {
            wheel.ang_vel = ci.normal.perp_dot(wheel.vel) / WHEEL_R;
            wheel.torque += ci.normal.perp_dot(wheel.force) * WHEEL_R;

            wheel.ang_vel += wheel.torque / WHEEL_INERTIA * dt;
            wheel.vel = ci.normal.perp() * wheel.ang_vel * WHEEL_R;
            b = true;
        }
    }
    if !b {
        wheel.ang_vel += wheel.torque / WHEEL_INERTIA * dt;
        wheel.vel += wheel.force / WHEEL_MASS * dt;
    }

    wheel.ang += wheel.ang_vel * dt;
    wheel.pos += wheel.vel * dt;

    // reset forces
    wheel.torque = 0.0;
    wheel.force = vec2(0.0, GRAVITY * WHEEL_MASS);
}

impl Bike {
    pub fn new(pos: Vec2) -> Bike {
        let pos = pos + vec2(0.0, -20.0);
        Bike {
            alive: true,
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
            prev_toggle_dir: false,
            jump: None,
        }
    }

    pub fn update(&mut self, dt: f32, level: &mut level::Level, input: &Input) {
        // toggle dir
        if input.toggle_dir && !self.prev_toggle_dir {
            self.dir = match self.dir {
                Direction::Right => Direction::Left,
                Direction::Left => Direction::Right,
            };
        }
        self.prev_toggle_dir = input.toggle_dir;
        self.dir_lerp = match self.dir {
            Direction::Right => (1.0_f32).min(self.dir_lerp + dt * 20.0),
            Direction::Left => (-1.0_f32).max(self.dir_lerp - dt * 20.0),
        };

        // apply forces
        // break
        let breaking = input.wheel == WheelInput::Break;
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
        if input.wheel == WheelInput::Accelerate && wheel.ang_vel * sign < MAX_SPEED {
            let torque = GAS * sign;
            wheel.torque += torque;
            self.frame.torque -= torque;
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

            // let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            self.frame.force -= force;
            self.frame.torque += force.perp_dot(arm);

            // friction
            // XXX: is this arm right?
            let arm = wheel.pos - self.frame.pos;

            let dv = self.frame.vel + self.frame.ang_vel * arm.perp() - wheel.vel;
            let force = dv * SUSPENSION_FRICTION;

            // let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            self.frame.force -= force;
            self.frame.torque += force.perp_dot(arm);
        }

        // start jump
        if let (Some(dir), None) = (input.jump, &self.jump) {
            self.jump = Some(Jump {
                dir: dir,
                time: 0.0,
                ang_vel: match dir {
                    Direction::Left => {
                        let res = self.frame.ang_vel.min(0.0);
                        self.frame.ang_vel -= JUMP_STRENGTH;
                        res
                    }
                    Direction::Right => {
                        let res = self.frame.ang_vel.max(0.0);
                        self.frame.ang_vel += JUMP_STRENGTH;
                        res
                    }
                },
            });
        }
        // jump
        if let Some(jump) = &mut self.jump {
            let t = jump.time;
            jump.time += dt;
            if t <= JUMP_DURATION && jump.time > JUMP_DURATION {
                match jump.dir {
                    Direction::Left => {
                        self.frame.ang_vel += JUMP_STRENGTH;
                        self.frame.ang_vel = self.frame.ang_vel.min(jump.ang_vel);
                    }
                    Direction::Right => {
                        self.frame.ang_vel -= JUMP_STRENGTH;
                        self.frame.ang_vel = self.frame.ang_vel.max(jump.ang_vel);
                    }
                };
            }
            if jump.time > JUMP_DURATION + JUMP_PAUSE {
                self.jump = None
            }
        }

        update_frame(&mut self.frame, dt);
        for wheel in self.wheels.iter_mut() {
            let ci = level.circle_collision(wheel.pos, WHEEL_R);
            if let Some(CollisionInfo {
                tpe: level::PolygonType::Lava,
                ..
            }) = ci
            {
                self.alive = false;
            }
            update_wheel(wheel, dt, ci);
        }

        // head collision
        let rot = Vec2::from_angle(self.frame.ang);
        let head = self.frame.pos + vec2(0.0, -21.0).rotate(rot);
        if let Some(_) = level.circle_collision(head, 4.4) {
            self.alive = false;
        }

        // pick up stars
        level.pickup_stars(head, 4.4);
        level.pickup_stars(self.frame.pos, 13.0);
        level.pickup_stars(self.wheels[0].pos, WHEEL_R);
        level.pickup_stars(self.wheels[1].pos, WHEEL_R);
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
        limb(-3, -15, -10, -9, 6.5, 6.8, c);
        // leg
        limb(-10, -9, -2, -3, 7.0, 4.0, c);
        limb(-2, -3, -1, 6, 4.0, 3.0, c);
        limb(-1, 6, 2, 6, 3.0, 2.0, c);

        // arm
        // TODO: clean up this mess
        // limb(-1, -15, 2, -8, 4.0, 3.0, c);
        // limb(2, -8, 10, -7, 3.0, 2.5, c);
        let ang = match &self.jump {
            Some(jump) => {
                let t = jump.time;
                let x = 0.0_f32.max(0.1_f32.powf(t) * 3.0 - 1.0) * (t * 10.0).min(1.0);
                if jump.dir == self.dir {
                    -x
                } else {
                    x
                }
            }
            None => 0.0,
        };
        let trans =
            trans * Affine2::from_scale_angle_translation(Vec2::ONE, ang, vec2(-1.0, -15.0));
        let t = |x: i32, y: i32| trans.transform_point2(vec2(x as f32, y as f32));
        let limb = |x1: i32, y1: i32, x2: i32, y2: i32, w: f32, v: f32, c: Color| {
            fx::draw_limb(t(x1, y1), t(x2, y2), w, v, c);
        };

        limb(0, 0, 3, 7, 4.0, 3.0, c);
        limb(3, 7, 11, 8, 3.0, 2.5, c);
    }
}
