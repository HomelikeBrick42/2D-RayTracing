static const int CHUNK_SIZE = 8;

struct Cell
{
    float3 color;
    uint solid;
}

struct Chunk
{
    Cell cells[CHUNK_SIZE * CHUNK_SIZE];
}

typealias Chunks = StructuredBuffer<Chunk, Std430DataLayout>;

struct Ray
{
    float2 origin;
    float2 direction;
}

struct Hit
{
    float distance;
    float3 color;
}

Optional<Hit> trace_chunk(Chunks chunks, int chunk_index, Ray ray, float max_distance)
{
    var hit : Hit;
    hit.distance = 0;
    hit.color = 1.0;

    let step_sizes = 1.0 / abs(ray.direction);
    let step_dir = int2(sign(ray.direction));
    var next_dist = (float2(step_dir) * 0.5 + 0.5 - fract(ray.origin)) / ray.direction;

    var voxel_pos = int2(floor(ray.origin));
    while (true)
    {
        let closest_dist = min(next_dist.x, next_dist.y);
        hit.distance += closest_dist;
        let step_axis = int2(next_dist == float2(closest_dist));
        let next_voxel_pos = voxel_pos + step_axis * step_dir;

        if (all(voxel_pos >= int2(0)) && all(voxel_pos < int2(CHUNK_SIZE)))
        {
            let index = int(voxel_pos.x) + int(voxel_pos.y) * CHUNK_SIZE;
            let cell = chunks[chunk_index].cells[index];

            if (cell.solid != 0)
            {
                if (all(next_voxel_pos >= int2(0)) && all(next_voxel_pos < int2(CHUNK_SIZE)))
                {
                    let next_index = int(next_voxel_pos.x) + int(next_voxel_pos.y) * CHUNK_SIZE;
                    let next_cell = chunks[chunk_index].cells[next_index];

                    if (next_cell.solid == 0)
                    {
                        hit.color *= 0.5;
                    }
                }
            }

            if (hit.distance >= max_distance)
            {
                hit.distance = max_distance;
                hit.color *= chunks[chunk_index].cells[index].color;
                return hit;
            }
        }
        else
        {
            return none;
        }

        voxel_pos = next_voxel_pos;
        next_dist -= closest_dist;
        next_dist += step_sizes * float2(step_axis);
    }
}
