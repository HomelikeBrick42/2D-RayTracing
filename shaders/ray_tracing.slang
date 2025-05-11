import chunk;

struct Camera
{
    float2 position;
    float half_height;
    float aspect;
}

[vk::binding(0, 0)]
uniform Camera camera;

[vk::binding(0, 1)]
Chunks chunks;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    float2 uv;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID)
{
    var out : VertexOutput;
    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);

    out.uv = float2(x, y) * 2.0 - 1.0;
    out.clip_position = float4(out.uv, 0.0, 1.0);

    return out;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;
    out.color = float4(0.0, 0.0, 0.0, 1.0);

    var world_position = in.uv * float2(camera.aspect, 1.0) * camera.half_height;
    world_position += camera.position;

    let camera_to_world = world_position - camera.position;
    let camera_to_world_distance = length(camera_to_world);

    var ray : Ray;
    ray.origin = camera.position;
    ray.direction = camera_to_world / camera_to_world_distance;
    if (let hit = trace_chunk(chunks, 0, ray, camera_to_world_distance))
    {
        out.color.xyz = hit.color;
    }

    return out;
}
