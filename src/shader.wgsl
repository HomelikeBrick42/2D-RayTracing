struct Camera {
    position: vec2<f32>,
    player_position: vec2<f32>,
    aspect_ratio: f32,
    vertical_view_height: f32,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct Chunk {
    position: vec2<f32>,
    size: vec2<u32>,
    blocks: array<Block>,
};

@group(1)
@binding(0)
var<storage, read> chunk: Chunk;

@group(1)
@binding(1)
var<storage, read> materials: array<Material>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(f32((vertex_index >> 0u) & 1u), f32((vertex_index >> 1u) & 1u));
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

struct Ray {
    origin: vec2<f32>,
    direction: vec2<f32>,
};

struct Hit {
    hit: bool,
    distance: f32,
    point: vec2<f32>,
    normal: vec2<f32>,
    material: u32,
};

const INVALID_MATERIAL: u32 = 4294967295u;
struct Material {
    color: vec3<f32>,
};

struct Block {
    material: u32,
};

fn trace(ray: Ray, max_distance: f32) -> Hit {
    var ray = ray;

    var hit: Hit;
    hit.hit = false;
    hit.distance = 0.0;

    // Ray trace relative to the chunk position
    let half_chunk_size = vec2<f32>(chunk.size) * 0.5;
    let ray_chunk_offset = chunk.position - half_chunk_size;
    ray.origin -= ray_chunk_offset;

    let ray_unit_step_size = vec2<f32>(
        sqrt(1.0 + (ray.direction.y / ray.direction.x) * (ray.direction.y / ray.direction.x)),
        sqrt(1.0 + (ray.direction.x / ray.direction.y) * (ray.direction.x / ray.direction.y)),
    );

    var map_check = vec2<i32>(floor(ray.origin));
    var ray_axis_length: vec2<f32>;
    var step: vec2<i32>;

    if ray.direction.x < 0.0 {
        step.x = -1;
        ray_axis_length.x = (ray.origin.x - f32(map_check.x)) * ray_unit_step_size.x;
    } else {
        step.x = 1;
        ray_axis_length.x = (f32(map_check.x + 1) - ray.origin.x) * ray_unit_step_size.x;
    }
    if ray.direction.y < 0.0 {
        step.y = -1;
        ray_axis_length.y = (ray.origin.y - f32(map_check.y)) * ray_unit_step_size.y;
    } else {
        step.y = 1;
        ray_axis_length.y = (f32(map_check.y + 1) - ray.origin.y) * ray_unit_step_size.y;
    }

    if ray_axis_length.x < ray_axis_length.y {
        hit.distance = ray_axis_length.x;
        hit.normal = vec2<f32>(f32(-step.x), 0.0);
    } else {
        hit.distance = ray_axis_length.y;
        hit.normal = vec2<f32>(0.0, f32(-step.y));
    }

    while !hit.hit && hit.distance <= max_distance {
        if all(vec2<i32>(0, 0) <= map_check) && all(map_check < vec2<i32>(chunk.size)) {
            hit.material = chunk.blocks[u32(map_check.x) + u32(map_check.y) * chunk.size.x].material;
            if hit.material != INVALID_MATERIAL {
                hit.hit = true;
                break;
            }
        }

        if ray_axis_length.x < ray_axis_length.y {
            map_check.x += step.x;
            hit.distance = ray_axis_length.x;
            hit.normal = vec2<f32>(f32(-step.x), 0.0);
            ray_axis_length.x += ray_unit_step_size.x;
        } else {
            map_check.y += step.y;
            hit.distance = ray_axis_length.y;
            hit.normal = vec2<f32>(0.0, f32(-step.y));
            ray_axis_length.y += ray_unit_step_size.y;
        }
    }
    hit.point = ray.origin + ray.direction * hit.distance;

    if hit.distance > max_distance {
        hit.hit = false;
    }

    // because the chunk position was subtracted at the start, its re-added here
    hit.point += ray_chunk_offset;
    return hit;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_world_coord = (in.uv - 0.5) * vec2<f32>(camera.aspect_ratio, 1.0) * camera.vertical_view_height + camera.position;
    let player_to_pixel = pixel_world_coord - camera.player_position;
    let pixel_distance = length(player_to_pixel);

    if pixel_distance < 0.1 {
        return vec4<f32>(1.0);
    }

    var ray: Ray;
    ray.origin = camera.player_position;
    ray.direction = player_to_pixel / pixel_distance;

    let hit = trace(ray, pixel_distance);
    if hit.hit {
        return vec4<f32>(materials[hit.material].color, 1.0);
    } else {
        return vec4<f32>(fract(pixel_world_coord), 0.0, 1.0);
    }
}
