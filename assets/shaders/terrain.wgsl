struct Uniforms {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexIn {
    @location(0) vertex_pos: vec3<f32>,
}

struct VertexOut {
    @builtin(position) vertex_pos: vec4<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut{
    var out: VertexOut;
    out.vertex_pos = uniforms.proj * uniforms.view * vec4<f32>(in.vertex_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(0.4, 0.3, 0.2, 1.0);
}
