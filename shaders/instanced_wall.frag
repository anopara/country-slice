// port of Bevy's PBR shader
// https://github.com/bevyengine/bevy/blob/97d8e4e1793ede3df8c77ed44736e800b38ff7a4/crates/bevy_pbr/src/render_graph/pbr_pipeline/pbr.frag

#version 450 core

in flat int instance_id;
in vec3 vertex_color;
in vec3 vertex_normal_ws;
in vec3 vertex_position_ws;
in vec3 curve_position_ws;

uniform mat4 view;
uniform mat4 projection;
uniform vec3 camera_position;

uniform sampler2D computeTexture;

uniform bool is_arch;

out vec4 FragColor;  

float random( int p ) {
    return fract(sin(dot(float(p), 311.7)) * 43758.5453);
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

#    define saturate(x) clamp(x, 0.0, 1.0)
const float PI = 3.141592653589793;

float pow5(float x) {
    float x2 = x * x;
    return x2 * x2 * x;
}

// Simple implementation, has precision problems when using fp16 instead of fp32
// see https://google.github.io/filament/Filament.html#listing_speculardfp16
float D_GGX(float roughness, float NoH, const vec3 h) {
    float oneMinusNoHSquared = 1.0 - NoH * NoH;
    float a = NoH * roughness;
    float k = roughness / (oneMinusNoHSquared + a * a);
    float d = k * k * (1.0 / PI);
    return d;
}

// distanceAttenuation is simply the square falloff of light intensity
// combined with a smooth attenuation at the edge of the light radius
//
// light radius is a non-physical construct for efficiency purposes,
// because otherwise every light affects every fragment in the scene
float getDistanceAttenuation(const vec3 posToLight, float inverseRadiusSquared) {
    #if 1
    float distanceSquare = dot(posToLight, posToLight);
    float factor = distanceSquare * inverseRadiusSquared;
    float smoothFactor = saturate(1.0 - factor * factor);
    float attenuation = smoothFactor * smoothFactor;
    return attenuation * 1.0 / max(distanceSquare, 1e-4);
    #else
    return 1000.0 / dot(posToLight, posToLight);
    #endif
}

// Visibility function (Specular G)
// V(v,l,a) = G(v,l,α) / { 4 (n⋅v) (n⋅l) }
// such that f_r becomes
// f_r(v,l) = D(h,α) V(v,l,α) F(v,h,f0)
// where
// V(v,l,α) = 0.5 / { n⋅l sqrt((n⋅v)^2 (1−α2) + α2) + n⋅v sqrt((n⋅l)^2 (1−α2) + α2) }
// Note the two sqrt's, that may be slow on mobile, see https://google.github.io/filament/Filament.html#listing_approximatedspecularv
float V_SmithGGXCorrelated(float roughness, float NoV, float NoL) {
    float a2 = roughness * roughness;
    float lambdaV = NoL * sqrt((NoV - a2 * NoV) * NoV + a2);
    float lambdaL = NoV * sqrt((NoL - a2 * NoL) * NoL + a2);
    float v = 0.5 / (lambdaV + lambdaL);
    return v;
}

// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
vec3 F_Schlick(const vec3 f0, float f90, float VoH) {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow5(1.0 - VoH);
}

float F_Schlick(float f0, float f90, float VoH) {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow5(1.0 - VoH);
}

vec3 fresnel(vec3 f0, float LoH) {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    float f90 = saturate(dot(f0, vec3(50.0 * 0.33)));
    return F_Schlick(f0, f90, LoH);
}

// Cook-Torrance approximation of the microfacet model integration using Fresnel law F to model f_m
// f_r(v,l) = { D(h,α) G(v,l,α) F(v,h,f0) } / { 4 (n⋅v) (n⋅l) }
vec3 specular(vec3 f0, float roughness, const vec3 h, float NoV, float NoL,
              float NoH, float LoH) {
    float D = D_GGX(roughness, NoH, h);
    float V = V_SmithGGXCorrelated(roughness, NoV, NoL);
    vec3 F = fresnel(f0, LoH);

    return (D * V) * F;
}

// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
float Fd_Burley(float roughness, float NoV, float NoL, float LoH) {
    float f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    float lightScatter = F_Schlick(1.0, f90, NoL);
    float viewScatter = F_Schlick(1.0, f90, NoV);
    return lightScatter * viewScatter * (1.0 / PI);
}

// From https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile
vec3 EnvBRDFApprox(vec3 f0, float perceptual_roughness, float NoV) {
    const vec4 c0 = { -1, -0.0275, -0.572, 0.022 };
    const vec4 c1 = { 1, 0.0425, 1.04, -0.04 };
    vec4 r = perceptual_roughness * c0 + c1;
    float a004 = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    vec2 AB = vec2(-1.04, 1.04) * a004 + r.zw;
    return f0 * AB.x + AB.y;
}

float perceptualRoughnessToRoughness(float perceptualRoughness) {
    // clamp perceptual roughness to prevent precision problems
    // According to Filament design 0.089 is recommended for mobile
    // Filament uses 0.045 for non-mobile
    float clampedPerceptualRoughness = clamp(perceptualRoughness, 0.089, 1.0);
    return clampedPerceptualRoughness * clampedPerceptualRoughness;
}

// luminance coefficients from Rec. 709.
// https://en.wikipedia.org/wiki/Rec._709
float luminance(vec3 v) {
    return dot(v, vec3(0.2126, 0.7152, 0.0722));
}

vec3 change_luminance(vec3 c_in, float l_out) {
    float l_in = luminance(c_in);
    return c_in * (l_out / l_in);
}

vec3 reinhard_luminance(vec3 color) {
    float l_old = luminance(color);
    float l_new = l_old / (1.0f + l_old);
    return change_luminance(color, l_new);
}

float arch_function(float h) {
    return 1.0 - exp(-5.0 * h);
    //return 1.0 - pow(1.0 - h, 8.0);
}


const float ALPHA = 0.14;
const float INV_ALPHA = 1.0 / ALPHA;
const float K = 2.0 / (PI * ALPHA);

float nrand( vec2 n )
{
	return fract(sin(dot(n.xy, vec2(12.9898, 78.233)))* 43758.5453);
}

float inv_error_function(float x)
{
	float y = log(1.0 - x*x);
	float z = K + 0.5 * y;
	return sqrt(sqrt(z*z - y * INV_ALPHA) - z) * sign(x);
}

float gaussian_rand( vec2 n , float seed)
{
	float t = fract( seed );
	float x = nrand( n + 0.07*t );
    
	return inv_error_function(x*2.0-1.0)*0.15 + 0.5;
}
  
void main()
{
    vec3 light_pos = vec3(4.0, 8.0, 4.0);
    vec4 light_color = vec4(1.0);
    float light_intensity = 200.0;
    float light_radius = 20.0;
    float perceptual_roughness = 0.9;
    float roughness = perceptualRoughnessToRoughness(perceptual_roughness);
    float metallic = 0.0;
    float reflectance = 0.1;


    //float r = fit01(random(instance_id), 0.15, 0.25);
    float r = gaussian_rand(vec2(instance_id+4), 0);
    r = clamp(r, 0.2, 1.0);
    r = fit01(r, 0.1, 0.35);
    vec4 output_color = vec4(vec3(r), 1.0);
    // Port from Bevy 0.5
    vec3 N = normalize(vertex_normal_ws);
    vec3 V = normalize(camera_position - vertex_position_ws);
    // Neubelt and Pettineo 2013, "Crafting a Next-gen Material Pipeline for The Order: 1886"
    float NdotV = max(dot(N, V), 1e-4);

    vec3 diffuseColor = output_color.rgb * (1.0 - metallic);

    // accumulate color
    vec3 light_accum = vec3(0.0);

    vec3 lightDir = light_pos - vertex_position_ws;
    vec3 L = normalize(lightDir);

    float inverseRadiusSquared = (1.0/light_radius) * (1.0/light_radius);
    float rangeAttenuation =
        getDistanceAttenuation(lightDir, inverseRadiusSquared) * light_intensity;

    vec3 H = normalize(L + V);
    float NoL = saturate(dot(N, L));
    float NoH = saturate(dot(N, H));
    float LoH = saturate(dot(L, H));

    // Remapping [0,1] reflectance to F0
    // See https://google.github.io/filament/Filament.html#materialsystem/parameterization/remapping
    vec3 F0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + output_color.rgb * metallic;

    vec3 specular = specular(F0, roughness, H, NdotV, NoL, NoH, LoH);
    vec3 diffuse = diffuseColor * Fd_Burley(roughness, NdotV, NoL, LoH);

    // Lout = f(v,l) Φ / { 4 π d^2 }⟨n⋅l⟩
    // where
    // f(v,l) = (f_d(v,l) + f_r(v,l)) * light_color
    // Φ is light intensity

    // our rangeAttentuation = 1 / d^2 multiplied with an attenuation factor for smoothing at the edge of the non-physical maximum light radius
    // It's not 100% clear where the 1/4π goes in the derivation, but we follow the filament shader and leave it out

    // See https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminanceEquation
    // TODO compensate for energy loss https://google.github.io/filament/Filament.html#materialsystem/improvingthebrdfs/energylossinspecularreflectance
    // light.color.rgb is premultiplied with light.intensity on the CPU
    light_accum +=
        ((diffuse + specular) * light_color.rgb) * (rangeAttenuation * NoL * 1.2);

    vec3 diffuse_ambient = EnvBRDFApprox(diffuseColor, 1.0, NdotV) * pow((1.0-NoL), 5.0) * 5.0;
    vec3 specular_ambient = EnvBRDFApprox(F0, perceptual_roughness, NdotV);

    output_color.rgb = light_accum;
    output_color.rgb += (diffuse_ambient + specular_ambient) * 0.075;// * AmbientColor.xyz * occlusion;
    //output_color.rgb += emissive.rgb * output_color.a;

    // tone_mapping
    //output_color.rgb = reinhard_luminance(output_color.rgb);

    if (!is_arch) {
        // sample compute texture
        // convert pos_ws to texture_uv
        // texture is from -10.0 to 10.0 in ws
        vec2 texture_uv = (curve_position_ws / 20.0 + 0.5).xz;
        float texture_color = texture(computeTexture, texture_uv).x; 

        float height_threshold = arch_function(texture_color);

        if (texture_color > 0.01 && curve_position_ws.y < height_threshold) { discard; }
    }

    FragColor = output_color;
}