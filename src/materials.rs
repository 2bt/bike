use macroquad::prelude::*;

pub struct Materials {
    pub wall_material: Material,
    pub font: Font,
}

impl Materials {
    pub fn load() -> Materials {

        let mut font = load_ttf_font_from_bytes(include_bytes!("../assets/Copilme.ttf")).unwrap();
        font.set_filter(FilterMode::Linear);

        Materials {
            wall_material: load_material(
                ShaderSource {
                    glsl_vertex: Some(
                        "#version 100
precision lowp float;
attribute vec3 position;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = position.xy;
}",
                    ),
                    glsl_fragment: Some(
                        "#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec2 f = fract(uv / 32.0);
    vec2 a = abs(f * 2.0 - vec2(1.0));
    float x = pow(max(a.x, a.y), 20.0);
    gl_FragColor = mix(
        vec4(0.11, 0.34, 0.22, 1.0),
        vec4(0.11, 0.4, 0.3, 1.0),
        x);
}",
                    ),
                    metal_shader: None,
                },
                Default::default(),
            )
            .unwrap(),
            font,
        }
    }
}
