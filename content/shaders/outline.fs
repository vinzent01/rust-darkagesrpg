#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

uniform sampler2D texture0;
uniform vec4 outlineColor;

out vec4 finalColor;

void main()
{
    vec4 tex = texture(texture0, fragTexCoord);

    if (tex.a > 0.0)
    {
        finalColor = tex * fragColor;
        return;
    }

    ivec2 texSize = textureSize(texture0, 0);
    ivec2 coord = ivec2(fragTexCoord * vec2(texSize));

    float alpha = 0.0;
    alpha += texelFetch(texture0, coord + ivec2( 1, 0), 0).a;
    alpha += texelFetch(texture0, coord + ivec2(-1, 0), 0).a;
    alpha += texelFetch(texture0, coord + ivec2( 0, 1), 0).a;
    alpha += texelFetch(texture0, coord + ivec2( 0,-1), 0).a;

    if (alpha > 0.0)
        finalColor = outlineColor;
    else
        finalColor = vec4(0.0);
}
