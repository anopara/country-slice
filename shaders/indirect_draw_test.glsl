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

// Find "exact" value where derivative of img starts changing along the segment
float find_t_change(vec3 p1, vec3 p2) {
    float t_out = 0.0;
    // subdivide the segment
    const uint SUBDIV = 100;
    for (int i=0; i<SUBDIV; i++) {
        float t1 = float(i) / float(SUBDIV);
        vec3 subdiv_p1 = mix(p1, p2, t1);

        float t2 = float(i+1) / float(SUBDIV);
        vec3 subdiv_p2 = mix(p1, p2, t2);

        // find where derivative starts changing & one of the elements is 0
        float h1 = position_ws_to_roadmask_value(subdiv_p1, imageSize(road_mask));
        float h2 = position_ws_to_roadmask_value(subdiv_p2, imageSize(road_mask));

        if (abs(h1-h2) > 0.001 && (h1 < 0.0001 || h2 < 0.0001)) {

            if (h1 < h2) { t_out = t1; } else { t_out = t2; }

            break;
        }
    }
    return t_out;
}

float arch_function(float h) {
    return 1.0 - pow(1.0 - h, 4.0);
}

vec3 curve_ws_to_arch_ws(vec3 curve_ws) {
    curve_ws.y = position_ws_to_roadmask_value(curve_ws, imageSize(road_mask));
    curve_ws.y = arch_function(curve_ws.y);//pow(curve_ws.y, 0.3);
    return curve_ws;
}

float length_arch(vec3 from, vec3 to) {
    // subdivide the segment
    const uint SUBDIV = 1000;
    float out_length = 0.0;

    for (int i=0; i<SUBDIV; i++) {

        vec3 subdiv_p1 = mix(from, to,  float(i) / float(SUBDIV));
        vec3 subdiv_p2 = mix(from, to, float(i+1) / float(SUBDIV));

        subdiv_p1 = curve_ws_to_arch_ws(subdiv_p1);
        subdiv_p2 = curve_ws_to_arch_ws(subdiv_p2);

        out_length += distance(subdiv_p1, subdiv_p2);
    }

    return out_length;
}

vec3 curve_ws_from_segment_u(vec3 from, vec3 to, float target_u, float seg_length) {
    // check for quick 1s and 0s
    if (target_u > 0.99) {
        return curve_ws_to_arch_ws(to);
    }

    if (target_u < 0.01) {
        return curve_ws_to_arch_ws(from);
    }

    // subdivide the segment
    const uint SUBDIV = 1000;
    float dist_traveled = 0.0;
    for (int i=0; i<SUBDIV+1; i++) {

        vec3 subdiv_p1 = mix(from, to,  float(i) / float(SUBDIV));
        vec3 subdiv_p2 = mix(from, to,  float(i+1) / float(SUBDIV));

        subdiv_p1 = curve_ws_to_arch_ws(subdiv_p1);
        subdiv_p2 = curve_ws_to_arch_ws(subdiv_p2);

        float current_u = dist_traveled / seg_length;

        if (current_u > target_u) {
            return subdiv_p2;
        }

        dist_traveled += distance(subdiv_p1, subdiv_p2);
    }


    return vec3(0.0, 0.0, 0.0);
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
    //float current_arch_length = 0;
    for (int i=0; i<curve_npt-1; i++) {
         // get curve segment positions
        vec3 p1 = curves[idx].positions[i].xyz;
        vec3 p2 = curves[idx].positions[i+1].xyz;

        // get heights
        float height_1 = position_ws_to_roadmask_value(p1, dims);
        float height_2 = position_ws_to_roadmask_value(p2, dims);

        if (abs(height_1-height_2) > 0.001  && (height_1 < 0.0001 || height_2 < 0.0001)) {
            // find exact positions where height starts to go up or down!
            float t = find_t_change(p1, p2);

            if (height_1 < height_2) {
                // curve starts to go up
                p1 = mix(p1, p2, t);
            } else {
                // curve is going down
                p2 = mix(p1, p2, t);
            }
            
        }

        if (height_1 > 0 || height_2 > 0 ) {

            // check segment length
            vec3 seg_p1 = vec3(p1.x, height_1, p1.z);
            vec3 seg_p2 = vec3(p2.x, height_2, p2.z);
            float seg_length = length_arch(seg_p1, seg_p2);

            // subdivide distance
            int total_segment_bricks = max(int(floor(seg_length / BRICK_WIDTH)), 1);

            total_bricks += total_segment_bricks;
        }
    } 

    uint instance_offset = atomicAdd(cmds[0].instanceCount, total_bricks); //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/atomicAdd.xhtml 

    
    // TODO: investigate stutterring
    
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

        if (abs(height_1-height_2) > 0.001  && (height_1 < 0.0001 || height_2 < 0.0001)) {
            // find exact positions where height starts to go up or down!
            float t = find_t_change(p1, p2);

            if (height_1 < height_2) {
                // arch curve starts to go up
                p1 = mix(p1, p2, t);
            } else {
                // arch curve is reaching its end
                p2 = mix(p1, p2, t);
            }
            
        }

        if (height_1 > 0 || height_2 > 0) {

            // check segment length
            vec3 seg_p1 = p1;
            vec3 seg_p2 = p2;
            float seg_length = length_arch(seg_p1, seg_p2);

            // subdivide distance
            int total_segment_bricks = max(int(floor(seg_length / BRICK_WIDTH)), 1);

            for (int k=0; k<total_segment_bricks; k++) {

                // TODO: debug my resampling of the curve along u value
                float u1 = float(k) / float(total_segment_bricks);
                float u2 = float(k+1) / float(total_segment_bricks);

                vec3 subseg_p1 = curve_ws_from_segment_u(seg_p1, seg_p2, u1, seg_length);
                vec3 subseg_p2 = curve_ws_from_segment_u(seg_p1, seg_p2, u2, seg_length);

                //if (false) {
                //    subseg_p1 = curve_ws_to_arch_ws(mix(seg_p1, seg_p2, 0.0)); 
                //    subseg_p2 = curve_ws_to_arch_ws(mix(seg_p1, seg_p2, 0.5)); 
                //}

                vec3 pivot = (subseg_p1+subseg_p2) / 2.0;

                float width = distance(subseg_p1, subseg_p2);

                vec3 s = vec3(width, 0.15, 0.25);

                vec3 x = normalize(subseg_p2-subseg_p1);
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
                    x.x, x.y, x.z, 0.0, 
                    y.x, y.y, y.z, 0.0, 
                    z.x, z.y, z.z, 0.0, 
                    0.0, 0.0, 0.0, 1.0
                );

                transforms[instance_offset] = translate * rotate * scale;
                instance_offset += 1;
            }

        }
        
    }
    
}  