use macroquad::prelude::*;

pub struct Materials {
    pub font: Font,
    pub wall_material: Material,
    pub lava_material: Material,
}

impl Materials {
    pub fn load() -> Materials {

        let mut font = load_ttf_font_from_bytes(include_bytes!("../assets/Copilme.ttf")).unwrap();
        font.set_filter(FilterMode::Linear);

        Materials {
            font,
            wall_material: load_material(
                ShaderSource::Glsl {
                    vertex: "#version 100
precision lowp float;
attribute vec3 position;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = position.xy;
}",
                    fragment: "#version 100
precision lowp float;
varying vec2 uv;
void main() {
    vec2 f = fract(uv / 32.0);
    vec2 a = abs(f * 2.0 - vec2(1.0));
    float x = pow(max(a.x, a.y), 20.0);
    gl_FragColor = mix(
        vec4(0.11, 0.34, 0.22, 1.0),
        vec4(0.11, 0.4, 0.3, 1.0),
        x);
}"
                },
                Default::default(),
            )
            .unwrap(),
            lava_material: load_material(
                ShaderSource::Glsl {
                    vertex: "#version 100
precision lowp float;
attribute vec3 position;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = position.xy;
}",
                    fragment: "#version 100
precision lowp float;
varying vec2 uv;
void main() {
    vec2 f = fract(uv / 32.0);
    vec2 a = abs(f * 2.0 - vec2(1.0));
    float x = pow(max(a.x, a.y), 20.0);
    gl_FragColor = mix(
        vec4(0.4, 0.2, 0.2, 1.0),
        vec4(0.5, 0.2, 0.2, 1.0),
        x);
}"
                },
                Default::default(),
            )
            .unwrap(),
        }
    }
}
