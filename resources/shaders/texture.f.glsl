#version 330 core

in vec2 TexCoord;
uniform sampler2D uTexture;

void main()
{
    gl_FragColor = texture(uTexture, TexCoord);
}