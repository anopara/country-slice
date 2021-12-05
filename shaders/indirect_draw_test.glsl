#version 430
layout(local_size_x = 1, local_size_y = 1) in;

// Same as the OpenGL defined struct: DrawElementsIndirectCommand
struct DrawCommand {
    uint count;         // Num elements (vertices)
    uint instanceCount; // Number of instances to draw (a.k.a primcount)
    uint firstIndex;    // Specifies a byte offset (cast to a pointer type) into the buffer bound to GL_ELEMENT_ARRAY_BUFFER to start reading indices from.
    uint baseVertex;    // Specifies a constant that should be added to each element of indices​ when chosing elements from the enabled vertex arrays.
    uint baseInstance;  // Specifies the base instance for use in fetching instanced vertex attributes.
};

// Command buffer backed by a Shader Storage Object Buffer (SSBO)
layout(std140, binding = 0) writeonly buffer draw_commands {
    DrawCommand cmds[];
};

layout (std430, binding=2) buffer transforms_buffer { 
    mat4 transforms[];
};

void main() {

    const uint idx = gl_LocalInvocationID.x;
    cmds[0].count = 312; // brick.glb vertex count
    cmds[0].instanceCount = 3;
    cmds[0].firstIndex = 0;   
    cmds[0].baseVertex = 0; 
    cmds[0].baseInstance = 0;   

    for (int i=0; i<3; i++) {
        transforms[i] = transpose(mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, float(i*1.5),
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ));
    }
} 