#version 450

layout(location = 0) in vec2 _uv;
layout(location = 1) in float curve_len;
layout(location = 0) out vec4 o_Target;

// From:
// Patricio Gonzalez Vivo: https://thebookofshaders.com/12/
//
vec2 random2( vec2 p ) {
    return fract(
        sin(
            vec2(
                dot(p, vec2(127.1, 311.7)),
                dot(p, vec2(269.5, 183.3))
            )
        ) * 43758.5453
    );
}

void main() {

    float BRICK_WIDTH = 0.2;
    float WALL_HEIGHT = 1.4;

    // Generate points in a grid

    
    // TODO: look here https://thebookofshaders.com/09/
    // https://www.youtube.com/watch?v=wBX6l8RqrT0&ab_channel=RyanPocock
    // I think I'm just gonna manually chop up the space, like I did for uving for the bricks
    // Can try this shader approach, alternatively, can do with geometry on CPU, and only add details in shader
    // need each brick with nice 0-1 UVS and also its ID for randomization, all that good stuff
    // also want to look into parallax mapping (https://learnopengl.com/Advanced-Lighting/Parallax-Mapping)
    // https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer

    vec2 point[5];
    point[0] = vec2(0.83,0.75);
    point[1] = vec2(0.60,0.07);
    point[2] = vec2(0.28,0.64);
    point[3] = vec2(0.31,0.26);
    point[4] = vec2(0.21,0.56);

 
    //float m_dist = 1.0;
    float thres = 0.1;
    float c = 0.0;

    for (int i = 0; i < 5; i++) {
        float dist = distance(_uv, point[i]);
        c = 1.0 - step(thres, dist);
    }

    float s = smoothstep(0.2,0.5, _uv.x*10.0) - smoothstep(0.5,0.8, _uv.x*10.0);

    //o_Target = vec4(s, s, s, 1.0);

    vec2 my_uv = _uv + random2(_uv)*0.01;

    float m_x = mod(my_uv.x * curve_len, BRICK_WIDTH);
    float m_y = mod(my_uv.y * WALL_HEIGHT, 0.15);
    float m = min(m_x, m_y);
    o_Target = vec4(m, m, m, 1.0) * 2.0;
}