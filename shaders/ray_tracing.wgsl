const VOXEL_SIZE: f32 = 1.0;
const MAXIMUM_TRAVERSAL_DISTANCE: i32 = 128;

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
var voxel_data: texture_3d<f32>;

@group(1) @binding(2)
var voxel_data_sampler: sampler;

struct Ray {
    origin: vec3f,
    direction: vec3f
};

struct HitRecord {
    normal: vec3f,
    pos: vec3f,
    t: f32,
    id: u32
};

fn schlick(cosine: f32, refractive_index: f32) -> f32 {
    let r0_ = (1.0 - refractive_index) / (1.0 + refractive_index);
    let r0 = r0_ * r0_;
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

fn ray_at(ray: Ray, t: f32) -> vec3f {
    return ray.origin + ray.direction * t;
}

fn voxel_traverse(ray: Ray) -> HitRecord {
    var record: HitRecord;
    let origin = ray.origin;
    let direction = normalize(ray.direction);

    let step = sign(direction + 0.000001);
    let stepi = vec3i(step);
    var current_voxel = vec3i(floor(origin / VOXEL_SIZE));
    let next_bound = vec3f(current_voxel + vec3i((step + vec3f(1.0)) * 0.5)) * VOXEL_SIZE;

    var t_max = (next_bound - origin) / direction;
    let t_delta = VOXEL_SIZE / direction * step;
    var i: i32 = 0;

    loop {
        if (t_max.x < t_max.y && t_max.x < t_max.z) {
            record.t = t_max.x;
            record.normal = vec3f(-step.x, 0.0, 0.0);
            t_max.x += t_delta.x;
            current_voxel.x += stepi.x;
        } else if (t_max.y < t_max.z) {
            record.t = t_max.y;
            record.normal = vec3f(0.0, -step.y, 0.0);
            t_max.y += t_delta.y;
            current_voxel.y += stepi.y;
        } else {
            record.t = t_max.z;
            record.normal = vec3f(0.0, 0.0, -step.z);
            t_max.z += t_delta.z;
            current_voxel.z += stepi.z;
        }

        record.id = u32(textureSample(voxel_data, voxel_data_sampler, vec3f(current_voxel)).r);
        if (record.id != 0u) {
            record.pos = ray_at(ray, record.t);
            break;
        }

        i += 1;
        if (i > MAXIMUM_TRAVERSAL_DISTANCE) {
            break;
        }
    }

    return record;
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
