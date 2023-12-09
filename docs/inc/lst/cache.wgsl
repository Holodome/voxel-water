let point = projection_matrix * prev_view_matrix * vec4f(pos, 1.0);
let p = point.xyz / point.w;
let puv1 = (p.xy + vec2f(1.0)) * 0.5;
let puv = vec2f(puv1.x, 1.0 - puv1.y);
let prev_normal = textureSample(prev_normal_tex, prev_tex_sampler, puv).rgb;
let prev_offset_id = textureSample(prev_offset_tex, prev_tex_sampler, puv).r;
let prev_mat_id = textureSample(prev_mat_tex, prev_tex_sampler, puv).r;
let prev_cache_tail = textureSample(prev_cache_tail_tex, prev_tex_sampler, puv).r;
let prev_color = textureSample(prev_color_tex, prev_tex_sampler, puv).rgb;
if result.material_id != 0.0 {
    if puv.x > 0.0 && puv.x < 1.0 &&
        puv.y > 0.0 && puv.y < 1.0 &&
        result.material_id == prev_mat_id && 
        distance(result.normal.xyz, prev_normal) < 0.1 && 
        result.offset_id == prev_offset_id
    {
        result.cache_tail = (1.0 - ALPHA) * prev_cache_tail;
        result.color = vec4f((ALPHA * result.color.xyz) + (1.0 - ALPHA) * prev_color, 1.0);
    } else {
        result.cache_tail = 1.0;
    }
}
