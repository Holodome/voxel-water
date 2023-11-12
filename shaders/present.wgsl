struct VertexInput {
    @location(0) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f
};

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = (in.uv * 2.0) - vec2f(1.0);
    out.pos = vec4f(pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(0) var samp: sampler;

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var uv = in.uv;
    uv.y = 1.0 - uv.y;
    return textureSample(tex, samp, uv);
}
