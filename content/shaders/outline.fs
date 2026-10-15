#version 100

varying mediump vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 outlineColor;

void main() {
    if (texture2D(Texture, uv).a > 0.0) {
        gl_FragColor = outlineColor;
    } else {
        discard;
    }
}
