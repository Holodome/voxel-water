const VOXEL_SIZE: f32 = 1.0;
const MAXIMUM_TRAVERSAL_DISTANCE: i32 = 128;
const MAX_BOUNCE_COUNT: i32 = 8;

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

struct Material {
    color: vec3f,
};

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

struct ScatterRecord {
    weight: vec3f,
    direction: vec3f
};

var<private> rng_state: u32;

fn xorshift32(state: u32) -> u32 {
    var x = state;
    x ^= x << 13u;
	x ^= x >> 17u;
	x ^= x << 5u;
    return x;
}

fn random_u32() -> u32 {
    let x = xorshift32(rng_state);
    rng_state = x;
    return x;
}

fn random_f32() -> f32 {
    let u = random_u32();
    return f32(u) * bitcast<f32>(0x2F800000u);
}

fn random_vec3f() -> vec3f {
    return vec3f(random_f32(), random_f32(), random_f32());
}

fn random_vec3f_range(low: f32, high: f32) -> vec3f {
    return vec3f(low) + random_vec3f() * (high - low);
}

fn random_in_unit_sphere() -> vec3f {
    var p: vec3f;
    loop {
        p = random_vec3f_range(-1.0, 1.0);
        if (dot(p, p) < 1.0) {
            break;
        }
    }
    return p;
}

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
    record.t = 1.0 / 0.0;
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

fn scatter(ray: Ray, hrec: HitRecord) -> ScatterRecord {
    var srec: ScatterRecord;

    var material: Material;
    material.color = vec3f(0.0, 1.0, 0.0);

    let target_vec = hrec.pos + hrec.normal + random_in_unit_sphere();
    srec.direction = target_vec - hrec.pos;
    srec.weight = material.color;

    return srec;
}

fn background(ray: Ray) -> vec3f {
    let unit_direction = normalize(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - t) * vec3f(1.0, 1.0, 1.0) + t * vec3f(0.5, 0.7, 1.0);
}

fn trace(ray_: Ray) -> vec3f {
    var throughput = vec3f(1.0);
    var ray = ray_;

    for (var i: i32 = 0; i < MAX_BOUNCE_COUNT; i += 1) {
        let hrec = voxel_traverse(ray);
        if (hrec.t != hrec.t) {
            return background(ray);
        }

        let srec = scatter(ray, hrec);
        throughput *= srec.weight;
        ray.origin = hrec.pos;
        ray.direction = srec.direction;
    }

    return throughput;
}

fn ray_color(ray: Ray) -> vec3f {
    return trace(ray);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    rng_state = xorshift32(bitcast<u32>(in.uv.x * 123456789.0 + in.uv.y));

    var ray: Ray;
    ray.origin = world.camera_at;
    ray.direction = world.camera_lower_left + in.uv.x * world.camera_horizontal 
        + in.uv.y * world.camera_vertical;

    return vec4f(ray_color(ray), 1.0);
}
