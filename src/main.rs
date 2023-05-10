use macroquad::prelude::*;
use std::f32::consts::PI;

mod fx;

const W: f32 = 480.0;
const H: f32 = 270.0;

struct Wall(Vec2, Vec2);

struct Body {
    mass: f32,
    inertia: f32,

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
            mass: 1.0,
            inertia: 1.0,
            pos: vec2(0.0, 0.0),
            ang: 0.0,
            vel: vec2(0.0, 0.0),
            ang_vel: 0.0,
            force: vec2(0.0, 0.0),
            torque: 0.0,
        };
    }
}

struct World {
    walls: Vec<Wall>,
    bike: Body,
    wheel1: Body,
    wheel2: Body,
}

#[derive(Debug)]
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

impl Body {
    fn reset_force(&mut self) {
        self.force = vec2(0.0, 0.0);
        self.force.y = GRAVITY * self.mass;
        self.torque = 0.0;
    }
    fn update(&mut self, dt: f32, ci: Option<CollisionInfo>) {
        if let Some(ci) = ci {
            self.pos += ci.normal * ci.dist;

            self.ang_vel = ci.normal.perp().dot(self.vel) / WHEEL_R;
            self.torque += ci.normal.perp().dot(self.force) * WHEEL_R;

            self.ang_vel += self.torque / self.inertia * dt;
            self.vel = ci.normal.perp() * self.ang_vel * WHEEL_R;
        } else {
            self.ang_vel += self.torque / self.inertia * dt;
            self.vel += self.force / self.mass * dt;
        }
        self.ang += self.ang_vel * dt;
        self.ang %= 2.0 * PI;

        self.pos += self.vel * dt;
    }
}

const GRAVITY: f32 = 100.0;
const WHEEL_R: f32 = 8.0;
const WHEEL_X: f32 = 17.0;
const WHEEL_Y: f32 = 12.0;
const SUSPENSION: f32 = 700.0;
const FRICTION: f32 = 40.0;
const MAX_SPEED: f32 = 44.0;
const GAS: f32 = 17000.0;

impl World {
    fn new() -> World {
        let mut world = World {
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
            bike: Body {
                mass: 20.0,
                inertia: 5000.0,
                pos: vec2(150.0, 100.0),
                ..Default::default()
            },
            wheel1: Body {
                inertia: 100.0,
                ..Body::default()
            },
            wheel2: Body {
                inertia: 100.0,
                ..Body::default()
            },
        };

        world.wheel1.pos = world.bike.pos + vec2(-WHEEL_X, WHEEL_Y);
        world.wheel2.pos = world.bike.pos + vec2(WHEEL_X, WHEEL_Y);

        world
    }

    fn check_wheel_collision(&self, wheel: &Body) -> Option<CollisionInfo> {
        let mut colli: Option<CollisionInfo> = None;
        for Wall(p, q) in self.walls.iter() {
            if let Some(ci) = circle_line_collision(wheel.pos, WHEEL_R, *p, *q) {
                colli = Some(match colli {
                    Some(cc) if cc.dist > ci.dist => cc,
                    _ => ci,
                })
            }
        }
        colli
    }

    fn update(&mut self, dt: f32) {
        // calc forces
        self.bike.reset_force();
        self.wheel1.reset_force();
        self.wheel2.reset_force();

        // move around
        if is_key_down(KeyCode::Right) {
            self.bike.force.x += 5000.0;
        }
        if is_key_down(KeyCode::Left) {
            self.bike.force.x -= 5000.0;
        }
        if is_key_down(KeyCode::Down) {
            self.bike.force.y += 5000.0;
        }
        if is_key_down(KeyCode::Up) {
            self.bike.force.y -= 5000.0;
        }

        // gas
        let do_gas = |bike: &mut Body, wheel: &mut Body, torque: f32| {
            wheel.torque += torque;
            bike.torque -= torque;
            // XXX: is this correct?
            // let arm = wheel.pos - bike.pos;
            // bike.force += torque * arm.perp() / arm.length_squared();
        };

        if is_key_down(KeyCode::C) && self.wheel1.ang_vel < MAX_SPEED {
            do_gas(&mut self.bike, &mut self.wheel1, GAS);
        }
        if is_key_down(KeyCode::X) && self.wheel1.ang_vel > -MAX_SPEED {
            do_gas(&mut self.bike, &mut self.wheel1, -GAS);
        }

        let rot = Vec2::from_angle(self.bike.ang);
        let do_suspension = |bike: &mut Body, wheel: &mut Body, t: Vec2| {
            let arm = t.rotate(rot);
            let force = (bike.pos + arm - wheel.pos) * SUSPENSION;

            let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            bike.force -= force;
            bike.torque += force.perp().dot(arm);

            // friction
            // XXX: is this arm right?
            let arm = wheel.pos - bike.pos;

            let dv = bike.vel + bike.ang_vel * arm.perp() - wheel.vel;
            let force = dv * FRICTION;

            let force = force * 1.0001_f32.powf(force.length());

            wheel.force += force;
            bike.force -= force;
            bike.torque += force.perp().dot(arm);
        };
        do_suspension(&mut self.bike, &mut self.wheel1, vec2(-WHEEL_X, WHEEL_Y));
        do_suspension(&mut self.bike, &mut self.wheel2, vec2(WHEEL_X, WHEEL_Y));

        self.bike.update(dt, None);
        let ci = self.check_wheel_collision(&self.wheel1);
        self.wheel1.update(dt, ci);
        let ci = self.check_wheel_collision(&self.wheel2);
        self.wheel2.update(dt, ci);
    }

    fn draw(&self) {
        clear_background(Color::new(0.05, 0.05, 0.05, 1.0));

        // walls
        for Wall(p, q) in self.walls.iter() {
            fx::draw_limb(*q, *p, 2.0, 2.0, DARKGREEN);
        }

        // wheels
        let c = Color::from_rgba(130, 130, 130, 255);
        fx::draw_wheel(self.wheel1.pos, self.wheel1.ang, WHEEL_R, c);
        fx::draw_wheel(self.wheel2.pos, self.wheel2.ang, WHEEL_R, c);

        let rot = Vec2::from_angle(self.bike.ang);
        // springs
        let c = Color::from_rgba(140, 80, 70, 255);
        let a = self.bike.pos + vec2(0.0, 9.0).rotate(rot);
        let b = self.bike.pos + vec2(12.0, -2.0).rotate(rot);
        fx::draw_limb(a, self.wheel1.pos, 3.0, 3.0, c);
        fx::draw_limb(b, self.wheel2.pos, 3.0, 3.0, c);

        // bike
        let c = Color::from_rgba(130, 130, 130, 255);
        let v = |x: i32, y: i32| self.bike.pos + vec2(x as f32, y as f32).rotate(rot);
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
            Color::from_rgba(80, 60, 60, 255),
        );

        // rider
        if true {
            let limb = |x1: i32, y1: i32, x2: i32, y2: i32, w: f32, v: f32, c: Color| {
                let p = self.bike.pos + vec2(x1 as f32, y1 as f32).rotate(rot);
                let q = self.bike.pos + vec2(x2 as f32, y2 as f32).rotate(rot);
                fx::draw_limb(p, q, w, v, c);
            };
            // head
            let q = self.bike.pos + vec2(0.0, -21.0).rotate(rot);
            draw_poly(q.x, q.y, 16, 4.5, self.bike.ang * (180.0 / PI), c);
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
