#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 0) out vec3 v_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);

    // I know that brick is within [-0.5; 0.5] bounding box
    v_color = Vertex_Position + 0.5;
}