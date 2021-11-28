#version 450 core
out vec4 FragColor;  

in vec3 ourColor;
in vec2 TexCoord;

uniform sampler2D ourTexture;
  
void main()
{
    vec2 uv = vec2(TexCoord.x, 1.0 - TexCoord.y);
    FragColor = texture(ourTexture, uv); //vec4(0.5, 1.0, 1.0, 1.0);
}