use macroquad::prelude::*;

mod bike;
mod fx;
mod level;
mod materials;

const W: f32 = 480.0;
const H: f32 = 270.0;

enum GameState {
    Playing,
    Over,
}

struct Game {
    state: GameState,
    time: f32,
    physics_time: f32,
    level: level::Level,
    bike: bike::Bike,
    materials: materials::Materials,
}

fn mix_color(a: Color, b: Color, x: f32) -> Color {
    Color::from_vec(a.to_vec() * (1.0 - x) + b.to_vec() * x)
}

impl Game {
    async fn new() -> Game {
        let level = level::Level::load("assets/level1.tmj").await.unwrap();
        let start = level.start;
        Game {
            state: GameState::Playing,
            time: 0.0,
            physics_time: 0.0,
            level: level,
            bike: bike::Bike::new(start),
            materials: materials::Materials::load(),
        }
    }

    fn update(&mut self) {
        // reset
        if is_key_down(KeyCode::Enter) {
            self.state = GameState::Playing;
            self.time = 0.0;
            self.physics_time = 0.0;
            self.bike = bike::Bike::new(self.level.start);
            self.level.reset_stars();
        }

        self.time += get_frame_time();

        match self.state {
            GameState::Playing => {
                let dt = 0.0002;
                while self.physics_time + dt < self.time {
                    self.physics_time += dt;
                    self.bike.update(dt, &mut self.level);

                    if !self.bike.alive {
                        self.state = GameState::Over;
                        self.time = 0.0;
                        break;
                    }
                }
            }
            GameState::Over => {}
        }
    }

    fn draw(&self) {
        let ratio = screen_width() / screen_height();
        let rect = if ratio > W / H {
            let w = H * ratio;
            Rect::new((w - W) * -0.5, 0.0, w, H)
        } else {
            let h = W / ratio;
            Rect::new(0.0, (h - H) * -0.5, W, h)
        };
        let mut cam = Camera2D::from_display_rect(rect);
        cam.zoom.y = cam.zoom.y.abs();
        cam.target = self.bike.frame.pos;
        set_camera(&cam);

        match self.state {
            GameState::Playing => {
                clear_background(Color::from_rgba(10, 12, 15, 255));
            }
            GameState::Over => {
                // flash
                let x = (self.time * 10.0).min(1.0);
                clear_background(mix_color(
                    Color::new(1.0, 1.0, 1.0, 1.0),
                    Color::from_rgba(10, 12, 15, 255),
                    x,
                ));
            }
        }

        self.level.draw(&self.materials);
        self.bike.draw();

        // labels
        let f = macroquad::text::camera_font_scale(8.0);
        let x = self.level.start.x;
        let mut y = self.level.start.y + 32.0;
        let mut txt = |text| {
            draw_text_ex(
                text,
                x,
                y,
                TextParams {
                    font_size: f.0,
                    font_scale: f.1,
                    ..TextParams::default()
                },
            );
            y += 8.0;
        };
        txt("[UP]    - accelerate");
        txt("[DOWN]  - break");
        txt("[SPACE] - toggle direction");
        txt("[ENTER] - reset position");


        cam.target.x = rect.w * 0.5;
        cam.target.y = rect.h * 0.5;
        set_camera(&cam);
        draw_text_ex(
            &format!("STARS: {}/{}", self.level.stars.len() - self.level.stars_left, self.level.stars.len()),
            6.0,
            10.0,
            TextParams {
                font_size: f.0,
                font_scale: f.1,
                ..TextParams::default()
            },
        );
    }
}

#[macroquad::main("Bike")]
async fn main() {
    let mut game = Game::new().await;
    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        game.update();
        game.draw();
        next_frame().await
    }
}
