struct VertexInput {
    @location(0) uv: vec2f
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f
};

@vertex 
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = (in.uv * 2.0) - vec2f(1.0);
    out.clip_position = vec4f(pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

struct WorldUniform {
    camera_at: vec3f,
    camera_lower_left: vec3f,
    camera_horizontal: vec3f,
    camera_vertical: vec3f,
};

@group(0) @binding(0)
var<uniform> world: WorldUniform;

@group(0) @binding(1)
var voxel_data: texture_3d<u32>;

@group(1) @binding(2)
var voxel_data_sampler: sampler;

struct Ray {
    origin: vec3f,
    direction: vec3f
};

fn ray_at(ray: Ray, t: f32) -> vec3f {
    return ray.origin + ray.direction * t;
}


fn ray_color(ray: Ray) -> vec3f {
    let unit_direction = normalize(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - t) * vec3f(1.0, 1.0, 1.0) + t * vec3f(0.5, 0.7, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var ray: Ray;
    ray.origin = world.camera_at;
    ray.direction = world.camera_lower_left + in.uv.x * world.camera_horizontal + in.uv.y * world.camera_vertical;

    return vec4f(ray_color(ray), 1.0);
}
