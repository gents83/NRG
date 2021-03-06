#version 450

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 nearPoint; 
layout(location = 2) in vec3 farPoint; 

//Output
layout(location = 0) out vec4 outColor;

#define MIN_GRID_DISTANCE 0.01
#define MAX_GRID_DISTANCE 20.
#define GRID_COLOR vec4(0.3, 0.3, 0.3, 1.0)


vec4 ComputeGridColor(vec3 pos, float scale, bool drawAxis) {
    vec2 coord = pos.xz * scale;
    vec2 derivative = fwidth(coord);
    vec2 grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    float line = min(grid.x, grid.y);
    float minimumz = min(derivative.y, 1);
    float minimumx = min(derivative.x, 1);
    vec4 color = GRID_COLOR;
    color.a = 1. - min(line, 1.);
    // z axis
    if(pos.x > -0.1 * minimumx && pos.x < 0.1 * minimumx)
        color.z = 1.0;
    // x axis
    if(pos.z > -0.1 * minimumz && pos.z < 0.1 * minimumz)
        color.x = 1.0;
    return color;
}

float ComputeDepth(vec3 pos) {
    vec4 clip_space_pos = pushConsts.proj * pushConsts.view * vec4(pos.xyz, 1.0);
    return (clip_space_pos.z / clip_space_pos.w);
}

float ComputeLinearDepth(vec3 pos) {
    vec4 clip_space_pos = pushConsts.proj * pushConsts.view * vec4(pos.xyz, 1.0);
    float clip_space_depth = (clip_space_pos.z / clip_space_pos.w) * 2.0 - 1.0; // put back between -1 and 1
    float linearDepth = (2.0 * MIN_GRID_DISTANCE * MAX_GRID_DISTANCE) / (MAX_GRID_DISTANCE + MIN_GRID_DISTANCE - clip_space_depth * (MAX_GRID_DISTANCE - MIN_GRID_DISTANCE)); // get linear value between 0.01 and 100
    return linearDepth / MAX_GRID_DISTANCE; // normalize
}

void main() {
    float t = -nearPoint.y / (farPoint.y - nearPoint.y);
    vec3 pos = nearPoint + t * (farPoint - nearPoint);

    gl_FragDepth = ComputeDepth(pos);

    float linearDepth = ComputeLinearDepth(pos);
    float fading = max(0, (0.5 - linearDepth));

    outColor = ComputeGridColor(pos, 1, true) * float(t > 0); // 1 mt resolution
    outColor += ComputeGridColor(pos, 10, true) * float(t > 0); // inner grid resolution
    outColor.a *= fading;
}