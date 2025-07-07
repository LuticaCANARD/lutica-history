// Bindings for texture and sampler
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// Vertex shader: Passes through vertex position and calculates texture coordinates.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index) / 2) * 2.0 - 1.0;
    let y = f32(i32(in_vertex_index) % 2) * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, -y, 0.0, 1.0);
    out.tex_coords = vec2<f32>(x * 0.5 + 0.5, y * 0.5 + 0.5);
    return out;
}

// Fragment shader: Applies a scanline effect to the texture.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Simple scanline effect
    let scanline = sin(in.clip_position.y * 300.0) * 0.1;
    color = color - vec4<f32>(scanline, scanline, scanline, 0.0);
    
    return color;
}