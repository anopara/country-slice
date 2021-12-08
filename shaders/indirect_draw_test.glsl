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

struct CurveData {
    uint points_count;
    uint pad0;
    uint pad1;
    uint pad2;
    vec4 positions[1000];
};

layout (std430, binding=3) buffer curves_buffer { 
    CurveData curves[];
};

layout(rgba32f) uniform image2D road_mask;

ivec2 ws_pos_to_pixel_coord(vec3 ws_pos, ivec2 img_dims) {
    // the img is from -10 to 10 in world space TODO: this should be a uniform coming from a struct on CPU side...
    vec2 texture_uv = (ws_pos / 20.0 + 0.5).xz;
    return ivec2(texture_uv.x * img_dims.x, texture_uv.y * img_dims.y);
}

float position_ws_to_roadmask_value(vec3 position, ivec2 dims) {
    ivec2 pixel_coord = ws_pos_to_pixel_coord(position, dims);
    return imageLoad(road_mask, pixel_coord).x;
}

void main() {

    float BRICK_WIDTH = 0.2;
    uint DEBUG_RESAMPLE = 3;

    ivec2 dims = imageSize(road_mask);
    const uint idx = gl_GlobalInvocationID.x;
    uint curve_npt = curves[idx].points_count;

    if (curve_npt < 2) {
        return;
    }

    // calculate how many bricks we need for the arch
    uint total_bricks = 0;
    for (int i=0; i<curve_npt-1; i++) {
         // get curve segment positions
        vec3 p1 = curves[idx].positions[i].xyz;
        vec3 p2 = curves[idx].positions[i+1].xyz;

        // get heights
        float height_1 = position_ws_to_roadmask_value(p1, dims);
        float height_2 = position_ws_to_roadmask_value(p2, dims);

        // check segment length
        vec3 seg_p1 = vec3(p1.x, height_1, p1.z);
        vec3 seg_p2 = vec3(p2.x, height_2, p2.z);
        float seg_length = distance(seg_p1, seg_p2);

        // subdivide distance
        int total_segment_bricks = int(ceil(seg_length / BRICK_WIDTH));

        // TODO: find exact positions where height starts to go up!
        if (height_1 > 0 || height_2 > 0) {
            total_bricks += total_segment_bricks+1; //TODO: hack to add +1, otherwise there is not enough allocations, needs investigation
        }
    }
    // TODO: have some kind of flickering or bricks.. is that a precision issue? :(
    // also, the program is being quite slow / stuttering now
 

    uint instance_offset = atomicAdd(cmds[0].instanceCount, total_bricks); //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/atomicAdd.xhtml 

    
    for (int i=0; i<curve_npt; i++) {

        // get curve segment positions
        vec3 p1 = curves[idx].positions[i].xyz;
        vec3 p2 = curves[idx].positions[i+1].xyz;
        vec3 curve_dir = p2-p1;

        // get heights
        float height_1 = position_ws_to_roadmask_value(p1, dims);
        float height_2 = position_ws_to_roadmask_value(p2, dims);

        //transforms[instance_offset] = transpose(mat4(
        //    0.01, 0.0, 0.0, p1.x,
        //    0.0, 1.0, 0.0, p1.y + pow(height_1, 0.3),
        //    0.0, 0.0, 0.01, p1.z,
        //    0.0, 0.0, 0.0, 1.0
        //));
        //instance_offset += 1;

        if (i >= curve_npt-1) {
            continue;
        }

        
        if (height_1 > 0 || height_2 > 0) {

            // check segment length
            vec3 seg_p1 = vec3(p1.x, pow(height_1, 0.3), p1.z);
            vec3 seg_p2 = vec3(p2.x, pow(height_2, 0.3), p2.z);
            float seg_length = distance(seg_p1, seg_p2);
            vec3 seg_dir = seg_p2-seg_p1;

            // subdivide distance
            int total_segment_bricks = int(ceil(seg_length / BRICK_WIDTH));

            for (int k=0; k<total_segment_bricks; k++) {
                vec3 subseg_p1 = seg_p1 + seg_dir * (float(k) / float(total_segment_bricks));
                vec3 subseg_p2 = seg_p1 + seg_dir * (float(k+1) / float(total_segment_bricks));

                vec3 pivot = (subseg_p1+subseg_p2) / 2.0;

                float subseg_w = seg_length / float(total_segment_bricks);

                vec3 s = vec3(subseg_w, 0.15, 0.25);

                vec3 x = normalize(seg_dir);
                vec3 z = normalize(cross(x, vec3(0.0, 1.0, 0.0)));
                vec3 y = normalize(cross(x, z));

                mat4 scale = transpose(mat4(
                    s.x, 0.0, 0.0, 0.0, 
                    0.0, s.y, 0.0, 0.0, 
                    0.0, 0.0, s.z, 0.0, 
                    0.0, 0.0, 0.0, 1.0
                ));

                mat4 translate = transpose(mat4(
                    1.0, 0.0, 0.0, pivot.x, 
                    0.0, 1.0, 0.0, pivot.y, 
                    0.0, 0.0, 1.0, pivot.z, 
                    0.0, 0.0, 0.0, 1.0
                ));

                mat4 rotate = mat4(
                    x.x, x.y, x.z, 0.0, //p1.x,
                    y.x, y.y, y.z, 0.0, //p1.y + pow(height_1, 0.3),
                    z.x, z.y, z.z, 0.0, //p1.z,
                    0.0, 0.0, 0.0, 1.0
                );

                transforms[instance_offset] = translate * rotate * scale;
                instance_offset += 1;
            }

        }
        
    }
    
    // ------------------------

    /*
    for (int i; i<(curve_npt-1); i++) {
        // get curve segment positions
        vec3 p1 = curves[idx].positions[i].xyz;
        vec3 p2 = curves[idx].positions[i+1].xyz;

        // get heights
        float height_1 = position_ws_to_roadmask_value(p1, dims);
        float height_2 = position_ws_to_roadmask_value(p2, dims);

        if (height_1 > 0 || height_2 > 0) {

            // check segment length
            vec3 seg_p1 = vec3(p1.x, pow(height_1, 0.3), p1.z);
            vec3 seg_p2 = vec3(p2.x, pow(height_2, 0.3), p2.z);
            float seg_length = distance(seg_p1, seg_p2);

            // subdivide distance
            int total_segment_bricks = DEBUG_RESAMPLE;//int(ceil(seg_length / BRICK_WIDTH));
            //vec3 subseg_dir = seg_p2-seg_p1;

            // calculate transforms
            for (int j; j<total_segment_bricks; j++) {

                //vec3 subseg_p1 = seg_p1 + subseg_dir * (float(j) / float(total_segment_bricks));
                //vec3 subseg_p2 = p1 + subseg_dir * (float(j+1) / float(total_segment_bricks));

                transforms[instance_offset+brick_count] = transpose(mat4(
                    0.1, 0.0, 0.0, p1.x,
                    0.0, 0.1, 0.0, p1.y,
                    0.0, 0.0, 0.1, p1.z,
                    0.0, 0.0, 0.0, 1.0
                ));

                brick_count += 1;

            }


        }
    }
    */
    
}  