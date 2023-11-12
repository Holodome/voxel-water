struct VertexInput {
    @location(0) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) pos: vec4f
};

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(vec4f(in.uv, 0.0, 1.0));
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(0) var samp: sampler;

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(tex, samp, in.pos.xy);
}
