struct Camera {
    position: vec2<f32>,
    player_position: vec2<f32>,
    aspect_ratio: f32,
    vertical_view_height: f32,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<storage, read> materials: array<Material>;

struct Chunk {
    position: vec2<f32>,
    size: vec2<u32>,
    blocks: array<Block>,
};

@group(1)
@binding(1)
var<storage, read> chunk: Chunk;

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

struct Material {
    color: vec3<f32>,
};

struct Block {
    material: u32,
};

fn trace(ray: Ray, max_distance: f32) -> Hit {
    var hit: Hit;
    hit.hit = false;
    return hit;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_world_coord = (in.uv - 0.5) * vec2<f32>(camera.aspect_ratio, 1.0) * camera.vertical_view_height + camera.position;
    let player_to_pixel = pixel_world_coord - camera.player_position;
    let pixel_distance = length(player_to_pixel);

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
