use macroquad::prelude::*;

mod bike;
mod fx;
mod level;
mod materials;

const W: f32 = 480.0;
const H: f32 = 270.0;

const LEVEL_FILES: [&'static str; 6] = [
    "level1.tmj", //
    "level2.tmj", //
    "level3.tmj", //
    "level4.tmj", //
    "level5.tmj", //
    "level6.tmj", //
];

#[derive(PartialEq, PartialOrd, Clone, Copy)]
struct LevelTime(u32);
impl LevelTime {
    fn new(t: f32) -> Self {
        Self((t * 100.0) as u32)
    }
    fn invalid() -> Self {
        Self(0xffffffff)
    }
}
impl std::fmt::Display for LevelTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0xffffffff {
            write!(f, "--:--:--")
        } else {
            write!(
                f,
                "{:0>2}:{:0>2}:{:0>2}",
                self.0 / (100 * 60),
                self.0 / 100 % 60,
                self.0 % 100,
            )
        }
    }
}

#[derive(PartialEq)]
enum GameState {
    LevelMenu,
    Playing,
    LevelCompleted,
    GameOver,
}

struct Game {
    canvas_size: Vec2,
    state: GameState,
    time: f32,
    physics_time: f32,
    level: level::Level,
    bike: bike::Bike,
    materials: materials::Materials,
    level_times: [LevelTime; LEVEL_FILES.len()],
    level_index: usize,
    running: bool,
}

fn mix_color(a: Color, b: Color, x: f32) -> Color {
    Color::from_vec(a.to_vec() * (1.0 - x) + b.to_vec() * x)
}

fn draw_text_mono(text: &str, x: f32, y: f32, params: TextParams) {
    let scale = (params.font_size as f32) * params.font_scale * 0.1;
    let mut x = x;
    for c in text.chars() {
        let o = match c {
            ':' => 1.0 * scale,
            '1' => 1.5 * scale,
            _ => 0.0,
        };
        draw_text_ex(&c.to_string(), x + o, y, params.clone());
        x += match c {
            ':' => 3.0 * scale,
            _ => 5.0 * scale,
        };
    }
}

impl Game {
    async fn new() -> Game {
        let mut game = Game {
            canvas_size: Default::default(),
            state: GameState::LevelMenu,
            time: 0.0,
            physics_time: 0.0,
            level: Default::default(),
            bike: Default::default(),
            materials: materials::Materials::load(),
            level_times: std::array::from_fn(|_| LevelTime::invalid()),
            level_index: 0,
            running: true,
        };
        game.reset_level().await;
        game.state = GameState::LevelMenu;
        game
    }

    async fn reset_level(&mut self) {
        self.state = GameState::Playing;
        self.time = 0.0;
        self.physics_time = 0.0;
        self.level = level::Level::load(&format!("assets/{}", LEVEL_FILES[self.level_index]))
            .await
            .unwrap();
        self.bike = bike::Bike::new(self.level.start);
    }

    async fn update(&mut self) {
        let ratio = screen_width() / screen_height();
        self.canvas_size = if ratio > W / H {
            vec2(H * ratio, H)
        } else {
            vec2(W, W / ratio)
        };

        let dt = get_frame_time();
        self.time += dt;

        // go back to level menu
        if is_key_pressed(KeyCode::Escape) {
            if self.state == GameState::LevelMenu {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.running = false;
                }
            } else {
                self.state = GameState::LevelMenu;
            }
        }

        // reset
        if is_key_pressed(KeyCode::Enter) {
            if self.state == GameState::LevelCompleted {
                self.state = GameState::LevelMenu;
            } else {
                self.reset_level().await;
            }
        }

        match self.state {
            GameState::LevelMenu => {
                if is_key_pressed(KeyCode::Up) && self.level_index > 0 {
                    self.level_index -= 1;
                }
                if is_key_pressed(KeyCode::Down) && self.level_index < self.level_times.len() - 1 {
                    self.level_index += 1;
                }
            }
            GameState::Playing => {
                self.level.update(dt);

                let input = bike::Input {
                    toggle_dir: is_key_down(KeyCode::Space),
                    wheel: match (is_key_down(KeyCode::Down), is_key_down(KeyCode::Up)) {
                        (true, false) => bike::WheelInput::Break,
                        (false, true) => bike::WheelInput::Accelerate,
                        _ => bike::WheelInput::None,
                    },
                    jump: match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
                        (true, false) => Some(bike::Direction::Left),
                        (false, true) => Some(bike::Direction::Right),
                        _ => None,
                    },
                };

                let dt = 0.0002;
                while self.physics_time + dt < self.time {
                    self.physics_time += dt;
                    self.bike.update(dt, &mut self.level, &input);

                    if self.level.stars_left == 0 {
                        self.state = GameState::LevelCompleted;
                        self.time = 0.0;

                        let t = &mut self.level_times[self.level_index];
                        let new_t = LevelTime::new(self.physics_time);
                        if new_t < *t {
                            *t = new_t;
                        }
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
        let mut cam = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            self.canvas_size.x,
            self.canvas_size.y,
        ));
        cam.zoom.y = cam.zoom.y.abs();
        cam.target = self.bike.frame.pos;
        set_camera(&cam);

        // background
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
        let tp_small = {
            let font_scale = macroquad::text::camera_font_scale(11.0);
            TextParams {
                font: Some(&self.materials.font),
                font_size: font_scale.0,
                font_scale: font_scale.1,
                ..TextParams::default()
            }
        };

        let tp_menu = {
            let font_scale = macroquad::text::camera_font_scale(20.0);
            TextParams {
                font: Some(&self.materials.font),
                font_size: font_scale.0,
                font_scale: font_scale.1,
                ..TextParams::default()
            }
        };

        let tp_big = {
            let font_scale = macroquad::text::camera_font_scale(40.0);
            TextParams {
                font: Some(&self.materials.font),
                font_size: font_scale.0,
                font_scale: font_scale.1,
                ..TextParams::default()
            }
        };

        // show star count and time
        cam.target = self.canvas_size * 0.5;
        set_camera(&cam);
        draw_text_mono(
            &format!(
                "{}/{}",
                self.level.stars.len() - self.level.stars_left,
                self.level.stars.len()
            ),
            5.0,
            12.0,
            tp_small.clone(),
        );
        draw_text_mono(
            &LevelTime::new(self.physics_time).to_string(),
            self.canvas_size.x - 42.0,
            12.0,
            tp_small.clone(),
        );

        cam.target = Vec2::ZERO;
        set_camera(&cam);

        match self.state {
            GameState::GameOver => {
                draw_text_ex("OUCH!", -70.0, -50.0, tp_big.clone());
            }
            GameState::LevelCompleted => {
                draw_text_ex("WELL DONE!", -117.0, -50.0, tp_big.clone());
            }
            GameState::LevelMenu => {
                draw_text_ex("BIKE", -52.0, -80.0, tp_big.clone());
                for (i, &t) in self.level_times.iter().enumerate() {
                    let y = -40.0 + (i as f32) * 24.0;
                    // cursor
                    if i == self.level_index {
                        let c = Color::new(0.8, 0.8, 0.3, 0.3);
                        draw_rectangle(-130.0, y - 19.0, 260.0, 24.0, c);
                    }
                    draw_text_ex("LEVEL", -120.0, y, tp_menu.clone());
                    draw_text_mono(&format!("{:>2}", i + 1), -70.0, y, tp_menu.clone());
                    draw_text_mono(&t.to_string(), 48.0, y, tp_menu.clone());
                }
            }
            _ => {}
        }
    }
}

#[macroquad::main("Bike")]
async fn main() {
    let mut game = Game::new().await;
    while game.running {
        game.update().await;
        game.draw();
        next_frame().await
    }
}
