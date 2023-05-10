use macroquad::prelude::*;
use std::f32::consts::PI;

mod fx;

const W: f32 = 480.0;
const H: f32 = 270.0;

const GRAVITY: f32 = 100.0;
const FRAME_MASS: f32 = 20.0;
const FRAME_INERTIA: f32 = 5000.0;
const WHEEL_MASS: f32 = 1.0;
const WHEEL_INERTIA: f32 = 100.0;
const WHEEL_R: f32 = 8.0;
const WHEEL_X: f32 = 17.0;
const WHEEL_Y: f32 = 12.0;
const SUSPENSION: f32 = 700.0;
const FRICTION: f32 = 40.0;
const MAX_SPEED: f32 = 50.0;
const GAS: f32 = 17000.0;

struct Wall(Vec2, Vec2);

struct Body {
    pos: Vec2,
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
    frame.ang %= 2.0 * PI;
    frame.pos += frame.vel * dt;
}

fn update_wheel(wheel: &mut Body, dt: f32, ci: Option<CollisionInfo>) {
    if let Some(ci) = ci {
        wheel.pos += ci.normal * ci.dist;

        wheel.ang_vel = ci.normal.perp().dot(wheel.vel) / WHEEL_R;
        wheel.torque += ci.normal.perp().dot(wheel.force) * WHEEL_R;

        wheel.ang_vel += wheel.torque / WHEEL_INERTIA * dt;
        wheel.vel = ci.normal.perp() * wheel.ang_vel * WHEEL_R;
    } else {
        wheel.ang_vel += wheel.torque / WHEEL_INERTIA * dt;
        wheel.vel += wheel.force / WHEEL_MASS * dt;
    }
    wheel.ang += wheel.ang_vel * dt;

    wheel.pos += wheel.vel * dt;
}

struct Bike {
    frame: Body,
    wheels: [Body; 2],
    breaking: bool,
    break_angles: [f32; 2],
}
impl Bike {
    fn new(pos: Vec2) -> Bike {
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
        }
    }

    fn update(&mut self, dt: f32, walls: &Vec<Wall>) {

        // calc forces
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
        self.breaking = breaking;
        if breaking {
            for (i, wheel) in self.wheels.iter_mut().enumerate() {
                let da = wheel.ang - self.frame.ang - self.break_angles[i];
                let dv = wheel.ang_vel - self.frame.ang_vel;

                let torque = da * 60000.0 + dv * 40000.0;
                wheel.torque -= torque;
                self.frame.torque += torque;
            }
        }
        else {
            self.wheels[0].ang %= 2.0 * PI;
            self.wheels[1].ang %= 2.0 * PI;
        }

        // gas
        let wheel = &mut self.wheels[0];
        if is_key_down(KeyCode::Up) && wheel.ang_vel < MAX_SPEED {
            wheel.torque += GAS;
            self.frame.torque -= GAS;
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
            self.frame.torque += force.perp().dot(arm);

            // friction
            // XXX: is this arm right?
            let arm = wheel.pos - self.frame.pos;

            let dv = self.frame.vel + self.frame.ang_vel * arm.perp() - wheel.vel;
            let force = dv * FRICTION;

            let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            self.frame.force -= force;
            self.frame.torque += force.perp().dot(arm);
        }

        update_frame(&mut self.frame, dt);
        for w in self.wheels.iter_mut() {
            let ci = check_wheel_collision(w, walls);
            update_wheel(w, dt, ci);
        }
    }

    fn draw(&self) {
        // wheels
        let c = Color::from_rgba(130, 130, 130, 255);
        for w in self.wheels.iter() {
            fx::draw_wheel(w.pos, w.ang, WHEEL_R, c);
        }

        let rot = Vec2::from_angle(self.frame.ang);
        // springs
        let c = Color::from_rgba(140, 80, 70, 255);
        let a = self.frame.pos + vec2(0.0, 9.0).rotate(rot);
        let b = self.frame.pos + vec2(12.0, -2.0).rotate(rot);
        fx::draw_limb(a, self.wheels[0].pos, 3.0, 3.0, c);
        fx::draw_limb(b, self.wheels[1].pos, 3.0, 3.0, c);

        // frame
        let v = |x: i32, y: i32| self.frame.pos + vec2(x as f32, y as f32).rotate(rot);
        fx::draw_polygon(
            &[
                v(2, -3),
                v(9, -9),
                v(14, -4),
                v(1, 11),
                v(-1, 11),
                v(-11, 0),
                v(-17, -2),
                v(-17, -7),
                v(-10, -7),
            ],
            Color::from_rgba(70, 60, 50, 255),
        );

        // rider
        if true {
            let c = Color::from_rgba(130, 130, 130, 255);
            let limb = |x1: i32, y1: i32, x2: i32, y2: i32, w: f32, v: f32, c: Color| {
                let p = self.frame.pos + vec2(x1 as f32, y1 as f32).rotate(rot);
                let q = self.frame.pos + vec2(x2 as f32, y2 as f32).rotate(rot);
                fx::draw_limb(p, q, w, v, c);
            };
            // head
            let q = self.frame.pos + vec2(0.0, -21.0).rotate(rot);
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
}

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

fn check_wheel_collision(wheel: &Body, walls: &Vec<Wall>) -> Option<CollisionInfo> {
    let mut colli: Option<CollisionInfo> = None;
    for Wall(p, q) in walls.iter() {
        if let Some(ci) = circle_line_collision(wheel.pos, WHEEL_R, *p, *q) {
            colli = Some(match colli {
                Some(cc) if cc.dist > ci.dist => cc,
                _ => ci,
            })
        }
    }
    colli
}

struct World {
    walls: Vec<Wall>,
    bike: Bike,
}

impl World {
    fn new() -> World {
        World {
            walls: vec![
                Wall(vec2(0.0, 0.0), vec2(0.0, H)),
                //
                Wall(vec2(0.0, 240.0), vec2(50.0, 245.0)),
                //
                Wall(vec2(50.0, 240.0), vec2(300.0, 250.0)),
                Wall(vec2(300.0, 250.0), vec2(450.0, 220.0)),
                Wall(vec2(450.0, 220.0), vec2(480.0, 70.0)),
                Wall(vec2(100.0, 140.0), vec2(200.0, 140.0)),
                //
                // Wall(vec2(0.0, 190.0), vec2(480.0, 190.0)),
            ],
            bike: Bike::new(vec2(150.0, 100.0)),
        }
    }

    fn update(&mut self, dt: f32) {

        if is_key_down(KeyCode::Enter) {
            *self = Self::new();
        }

        self.bike.update(dt, &self.walls);
    }

    fn draw(&self) {
        clear_background(Color::new(0.05, 0.05, 0.05, 1.0));

        for Wall(p, q) in self.walls.iter() {
            fx::draw_limb(*q, *p, 2.0, 2.0, DARKGREEN);
        }

        self.bike.draw();
    }
}

#[macroquad::main("Bike")]
async fn main() {
    let mut world = World::new();

    let mut time = 0.0;
    let mut physics_time = 0.0;

    // let canvas = render_target(W as u32, H as u32);
    // canvas.texture.set_filter(FilterMode::Nearest);
    // let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, W, H));
    // cam.render_target = Some(canvas);

    loop {
        // update

        time += get_frame_time();
        let dt = 0.0001;
        while physics_time + dt < time {
            physics_time += dt;
            world.update(dt);
        }

        // draw
        let ratio = screen_width() / screen_height();
        let cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, W, W / ratio));
        set_camera(&cam);
        world.draw();

        // // draw canvas
        // let ratio = screen_width() / screen_height();
        // let rect = if ratio > W / H {
        //     let w = H * ratio;
        //     Rect::new((w - W) * -0.5, 0.0, w, H)
        // } else {
        //     let h = W / ratio;
        //     Rect::new(0.0, (h - H) * -0.5, W, h)
        // };
        // let mut cam = Camera2D::from_display_rect(rect);
        // cam.zoom.y *= -1.0;
        // set_camera(&cam);
        // draw_texture(canvas.texture, 0.0, 0.0, WHITE);

        next_frame().await
    }
}
