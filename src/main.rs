use macroquad::prelude::*;

mod bike;
mod fx;
mod level;
mod materials;

const W: f32 = 480.0;
const H: f32 = 270.0;

enum GameState {
    Playing,
    LevelCompleted,
    GameOver,
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

        let dt = get_frame_time();
        self.time += dt;

        match self.state {
            GameState::Playing => {
                self.level.update(dt);

                let dt = 0.0002;
                while self.physics_time + dt < self.time {
                    self.physics_time += dt;
                    self.bike.update(dt, &mut self.level);

                    if self.level.stars_left == 0 {
                        self.state = GameState::LevelCompleted;
                        self.time = 0.0;
                        break;
                    }
                    if !self.bike.alive {
                        self.state = GameState::GameOver;
                        self.time = 0.0;
                        break;
                    }
                }
            }
            _ => {}
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
            GameState::GameOver => {
                // flash
                let x = (self.time * 5.0).min(1.0);
                clear_background(mix_color(
                    Color::new(1.0, 0.0, 0.0, 1.0),
                    Color::from_rgba(10, 12, 15, 255),
                    x,
                ));
            }
            _ => {
                clear_background(Color::from_rgba(10, 12, 15, 255));
            }
        }

        self.level.draw(&self.materials);
        self.bike.draw();

        // labels
        let font_scale = macroquad::text::camera_font_scale(10.0);
        let text_params = TextParams {
            font: Some(&self.materials.font),
            font_size: font_scale.0,
            font_scale: font_scale.1,
            ..TextParams::default()
        };

        {
            let x = self.level.start.x - 16.0;
            let mut y = self.level.start.y + 64.0;
            let mut txt = |left, right| {
                draw_text_ex(left, x, y, text_params.clone());
                draw_text_ex(right, x + 64.0, y, text_params.clone());
                y += 16.0;
            };
            txt("[UP]   ", "accelerate");
            txt("[DOWN] ", "break");
            txt("[SPACE]", "toggle direction");
            txt("[ENTER]", "reset position");
        }

        cam.target = rect.size() * 0.5;
        set_camera(&cam);
        draw_text_ex(
            &format!(
                "STARS: {}/{}",
                self.level.stars.len() - self.level.stars_left,
                self.level.stars.len()
            ),
            6.0,
            10.0,
            text_params.clone(),
        );

        // show time
        cam.target = vec2(-rect.w, rect.h) * 0.5;
        set_camera(&cam);
        let t = (self.physics_time * 100.0) as u32;
        let str = format!(
            "{:0>2}:{:0>2}:{:0>2}",
            t / (100 * 60),
            t / 100 % 60,
            t % 100,
        );
        let mut x = -40.0;
        // make it a bit more monospacy
        for c in str.chars() {
            let o = match c {
                ':' => 1.0,
                '1' => 1.5,
                _ => 0.0,
            };
            draw_text_ex(&c.to_string(), x + o, 10.0, text_params.clone());
            x += match c {
                ':' => 3.0,
                _ => 5.0,
            };
        }

        if let GameState::LevelCompleted = self.state {
            cam.target = Vec2::ZERO;
            set_camera(&cam);
            let msg = "WELL DONE!";
            let fs = macroquad::text::camera_font_scale(30.0);
            let tp = TextParams {
                font: Some(&self.materials.font),
                font_size: fs.0,
                font_scale: fs.1,
                ..TextParams::default()
            };
            draw_text_ex(msg, -75.0, -50.0, tp);
        }
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
