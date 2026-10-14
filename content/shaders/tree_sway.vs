#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;

uniform mat4 mvp;

// tempo em segundos
uniform float time;

// força do vento
uniform float windStrength;

// altura total da textura
uniform float textureHeight;

void main()
{
    vec3 pos = vertexPosition;

    // fator de influência baseado na altura do pixel
    // tronco (baixo) = 0, copa (topo) = 1
    float heightFactor = pos.y / textureHeight;

    // onda senoidal do vento
    float sway = sin(time + pos.y * 0.05) * windStrength;

    // desloca apenas no eixo X
    pos.x += sway * heightFactor;

    gl_Position = mvp * vec4(pos, 1.0);

    fragTexCoord = vertexTexCoord;
    fragColor = vertexColor;
}