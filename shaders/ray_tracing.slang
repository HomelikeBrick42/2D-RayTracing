struct Camera
{
    float2 position;
    float half_height;
    float aspect;
}

[vk::binding(0, 0)]
uniform Camera camera;

static const int CHUNK_SIZE = 2;

struct Cell
{
    float3 color;
}

struct Chunk
{
    Cell cells[CHUNK_SIZE * CHUNK_SIZE];
    float2 position;
}

[vk::binding(0, 1)]
StructuredBuffer<Chunk, Std430DataLayout> chunks;

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

    {
        let chunk = chunks[0];
        let chunk_relative_position = int2(floor(world_position - chunk.position));
        if ((chunk_relative_position.x >= 0) && (chunk_relative_position.x < CHUNK_SIZE) && (chunk_relative_position.y >= 0) && (chunk_relative_position.y < CHUNK_SIZE))
        {
            int index = chunk_relative_position.x + chunk_relative_position.y * CHUNK_SIZE;
            out.color.xyz = chunk.cells[index].color;
        }
        else
        {
            out.color.xz = 1.0;
        }
    }

    return out;
}
