// use anyhow::Context;
// use clap::Parser;
// use glam::Vec3;
// use oc_geo::tile::TileXy;
// use oc_root::GEO_PIXELS_PER_TILE;
// use oc_world::reader::MapReader;
// use serde_json::json;
// use std::fs::File;
// use std::io::{BufWriter, Write};
// use std::path::PathBuf;

// #[derive(Parser, Debug, Clone)]
// #[command(version, about, long_about = None)]
// struct Args {
//     /// World path
//     #[clap()]
//     path: PathBuf,

//     /// glb fil output path
//     #[clap()]
//     output: PathBuf,
// }

// fn main() -> Result<(), anyhow::Error> {
//     let args = Args::parse();
//     let map = MapReader::new(&args.path);
//     let map = map.context(format!("Read map from {}", args.path.display()))?;
//     // FIXME BS NOW: permit arg for region
//     let tiles = map.tiles().context("Read tiles")?;
//     let vertices: Vec<Vec3> = tiles
//         .iter()
//         .map(|tile| {
//             let xy: TileXy = tile.i.into();
//             let x = xy.0.0 * GEO_PIXELS_PER_TILE;
//             let y = xy.0.1 * GEO_PIXELS_PER_TILE;
//             let z = tile.z;
//             Vec3::new(x as f32, y as f32, z as f32)
//         })
//         .collect();
//     let indices: Vec<u32> = tiles.iter().map(|t| t.i.0 as u32).collect();

//     vec3_to_glb(&vertices, &indices, &args.output).context("Convert vec3")?;
//     Ok(())
// }

// /// Pad a byte vec to a multiple of 4, filling with `pad_byte`.
// fn pad4(data: &mut Vec<u8>, pad_byte: u8) {
//     while data.len() % 4 != 0 {
//         data.push(pad_byte);
//     }
// }

// /// Write a u32 in little-endian to a writer.
// fn write_u32<W: Write>(w: &mut W, v: u32) -> std::io::Result<()> {
//     w.write_all(&v.to_le_bytes())
// }

// pub fn vec3_to_glb(
//     vertices: &[Vec3],
//     indices: &[u32],
//     output_path: &PathBuf,
// ) -> std::io::Result<()> {
//     // ── 1. Build the binary chunk (BIN) ──────────────────────────────────────
//     //
//     //  Layout inside the single buffer:
//     //    [0 .. index_byte_len)            → u32 indices
//     //    [index_byte_len_padded .. end)   → f32×3 positions
//     //
//     let mut index_bytes: Vec<u8> = Vec::with_capacity(indices.len() * 4);
//     for &i in indices {
//         index_bytes.extend_from_slice(&i.to_le_bytes());
//     }
//     pad4(&mut index_bytes, 0x00);
//     let index_byte_len_padded = index_bytes.len(); // offset where positions start

//     let mut vertex_bytes: Vec<u8> = Vec::with_capacity(vertices.len() * 12);
//     for v in vertices {
//         vertex_bytes.extend_from_slice(&v.x.to_le_bytes());
//         vertex_bytes.extend_from_slice(&v.y.to_le_bytes());
//         vertex_bytes.extend_from_slice(&v.z.to_le_bytes());
//     }

//     let mut bin_chunk: Vec<u8> = index_bytes;
//     bin_chunk.extend_from_slice(&vertex_bytes);
//     pad4(&mut bin_chunk, 0x00); // BIN chunk must be 4-byte aligned

//     // ── 2. Compute bounding box (required by glTF spec on POSITION) ──────────
//     let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
//     for v in vertices {
//         let a = [v.x, v.y, v.z];
//         for i in 0..3 {
//             min[i] = min[i].min(a[i]);
//             max[i] = max[i].max(a[i]);
//         }
//     }

//     // ── 3. Build the JSON chunk ───────────────────────────────────────────────
//     //
//     //  GLB buffer has no URI — it refers implicitly to the embedded BIN chunk.
//     //
//     let gltf_json = json!({
//         "asset": { "version": "2.0", "generator": "vec3-to-glb (Rust)" },
//         "scene": 0,
//         "scenes": [{ "nodes": [0] }],
//         "nodes":  [{ "mesh": 0 }],
//         "meshes": [{
//             "name": "mesh",
//             "primitives": [{
//                 "attributes": { "POSITION": 1 },
//                 "indices": 0,
//                 "mode": 4        // TRIANGLES
//             }]
//         }],
//         "accessors": [
//             {
//                 // 0 — indices
//                 "bufferView":    0,
//                 "byteOffset":    0,
//                 "componentType": 5125,   // UNSIGNED_INT
//                 "count":         indices.len(),
//                 "type":          "SCALAR"
//             },
//             {
//                 // 1 — positions
//                 "bufferView":    1,
//                 "byteOffset":    0,
//                 "componentType": 5126,   // FLOAT
//                 "count":         vertices.len(),
//                 "type":          "VEC3",
//                 "min": [min[0], min[1], min[2]],
//                 "max": [max[0], max[1], max[2]]
//             }
//         ],
//         "bufferViews": [
//             {
//                 // 0 — index data
//                 "buffer":     0,
//                 "byteOffset": 0,
//                 "byteLength": indices.len() * 4,
//                 "target":     34963      // ELEMENT_ARRAY_BUFFER
//             },
//             {
//                 // 1 — vertex data
//                 "buffer":     0,
//                 "byteOffset": index_byte_len_padded,
//                 "byteLength": vertices.len() * 12,
//                 "byteStride": 12,
//                 "target":     34962      // ARRAY_BUFFER
//             }
//         ],
//         "buffers": [{
//             // No "uri" field → GLB embedded buffer
//             "byteLength": bin_chunk.len()
//         }]
//     });

//     let mut json_bytes: Vec<u8> = serde_json::to_string(&gltf_json).unwrap().into_bytes();
//     pad4(&mut json_bytes, 0x20); // JSON chunk pads with spaces (0x20)

//     // ── 4. Assemble the GLB file ──────────────────────────────────────────────
//     //
//     //  GLB layout:
//     //  ┌─────────────────────────────────┐
//     //  │  GLB header        (12 bytes)   │
//     //  ├─────────────────────────────────┤
//     //  │  Chunk 0 header    ( 8 bytes)   │  type = 0x4E4F534A ("JSON")
//     //  │  Chunk 0 data      (variable)   │
//     //  ├─────────────────────────────────┤
//     //  │  Chunk 1 header    ( 8 bytes)   │  type = 0x004E4942 ("BIN\0")
//     //  │  Chunk 1 data      (variable)   │
//     //  └─────────────────────────────────┘
//     //
//     const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
//     const GLB_VERSION: u32 = 2;
//     const CHUNK_TYPE_JSON: u32 = 0x4E4F534A;
//     const CHUNK_TYPE_BIN: u32 = 0x004E4942;

//     let total_len: u32 = (12                        // GLB header
//         + 8 + json_bytes.len()                      // JSON chunk
//         + 8 + bin_chunk.len()) as u32; // BIN  chunk

//     let file = File::create(output_path)?;
//     let mut w = BufWriter::new(file);

//     // GLB header
//     write_u32(&mut w, GLB_MAGIC)?;
//     write_u32(&mut w, GLB_VERSION)?;
//     write_u32(&mut w, total_len)?;

//     // Chunk 0 — JSON
//     write_u32(&mut w, json_bytes.len() as u32)?;
//     write_u32(&mut w, CHUNK_TYPE_JSON)?;
//     w.write_all(&json_bytes)?;

//     // Chunk 1 — BIN
//     write_u32(&mut w, bin_chunk.len() as u32)?;
//     write_u32(&mut w, CHUNK_TYPE_BIN)?;
//     w.write_all(&bin_chunk)?;

//     println!(
//         "Written: {}  ({} B, {} vertices, {} indices)",
//         output_path.display(),
//         total_len,
//         vertices.len(),
//         indices.len()
//     );
//     Ok(())
// }
// Cargo.toml:
// serde_json = "1"

use serde_json::json;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Pad a byte vec to a multiple of 4, filling with `pad_byte`.
fn pad4(data: &mut Vec<u8>, pad_byte: u8) {
    while !data.len().is_multiple_of(4) {
        data.push(pad_byte);
    }
}

/// Write a u32 in little-endian to a writer.
fn write_u32<W: Write>(w: &mut W, v: u32) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

pub fn vec3_to_glb(vertices: &[Vec3], indices: &[u32], output_path: &str) -> std::io::Result<()> {
    // ── 1. Build the binary chunk (BIN) ──────────────────────────────────────
    //
    //  Layout inside the single buffer:
    //    [0 .. index_byte_len)            → u32 indices
    //    [index_byte_len_padded .. end)   → f32×3 positions
    //
    let mut index_bytes: Vec<u8> = Vec::with_capacity(indices.len() * 4);
    for &i in indices {
        index_bytes.extend_from_slice(&i.to_le_bytes());
    }
    pad4(&mut index_bytes, 0x00);
    let index_byte_len_padded = index_bytes.len(); // offset where positions start

    let mut vertex_bytes: Vec<u8> = Vec::with_capacity(vertices.len() * 12);
    for v in vertices {
        vertex_bytes.extend_from_slice(&v.x.to_le_bytes());
        vertex_bytes.extend_from_slice(&v.y.to_le_bytes());
        vertex_bytes.extend_from_slice(&v.z.to_le_bytes());
    }

    let mut bin_chunk: Vec<u8> = index_bytes;
    bin_chunk.extend_from_slice(&vertex_bytes);
    pad4(&mut bin_chunk, 0x00); // BIN chunk must be 4-byte aligned

    // ── 2. Compute bounding box (required by glTF spec on POSITION) ──────────
    let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in vertices {
        let a = [v.x, v.y, v.z];
        for i in 0..3 {
            min[i] = min[i].min(a[i]);
            max[i] = max[i].max(a[i]);
        }
    }

    // ── 3. Build the JSON chunk ───────────────────────────────────────────────
    //
    //  GLB buffer has no URI — it refers implicitly to the embedded BIN chunk.
    //
    let gltf_json = json!({
        "asset": { "version": "2.0", "generator": "vec3-to-glb (Rust)" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes":  [{ "mesh": 0 }],
        "meshes": [{
            "name": "mesh",
            "primitives": [{
                "attributes": { "POSITION": 1 },
                "indices": 0,
                "mode": 4        // TRIANGLES
            }]
        }],
        "accessors": [
            {
                // 0 — indices
                "bufferView":    0,
                "byteOffset":    0,
                "componentType": 5125,   // UNSIGNED_INT
                "count":         indices.len(),
                "type":          "SCALAR"
            },
            {
                // 1 — positions
                "bufferView":    1,
                "byteOffset":    0,
                "componentType": 5126,   // FLOAT
                "count":         vertices.len(),
                "type":          "VEC3",
                "min": [min[0], min[1], min[2]],
                "max": [max[0], max[1], max[2]]
            }
        ],
        "bufferViews": [
            {
                // 0 — index data
                "buffer":     0,
                "byteOffset": 0,
                "byteLength": indices.len() * 4,
                "target":     34963      // ELEMENT_ARRAY_BUFFER
            },
            {
                // 1 — vertex data
                "buffer":     0,
                "byteOffset": index_byte_len_padded,
                "byteLength": vertices.len() * 12,
                "byteStride": 12,
                "target":     34962      // ARRAY_BUFFER
            }
        ],
        "buffers": [{
            // No "uri" field → GLB embedded buffer
            "byteLength": bin_chunk.len()
        }]
    });

    let mut json_bytes: Vec<u8> = serde_json::to_string(&gltf_json).unwrap().into_bytes();
    pad4(&mut json_bytes, 0x20); // JSON chunk pads with spaces (0x20)

    // ── 4. Assemble the GLB file ──────────────────────────────────────────────
    //
    //  GLB layout:
    //  ┌─────────────────────────────────┐
    //  │  GLB header        (12 bytes)   │
    //  ├─────────────────────────────────┤
    //  │  Chunk 0 header    ( 8 bytes)   │  type = 0x4E4F534A ("JSON")
    //  │  Chunk 0 data      (variable)   │
    //  ├─────────────────────────────────┤
    //  │  Chunk 1 header    ( 8 bytes)   │  type = 0x004E4942 ("BIN\0")
    //  │  Chunk 1 data      (variable)   │
    //  └─────────────────────────────────┘
    //
    const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
    const GLB_VERSION: u32 = 2;
    const CHUNK_TYPE_JSON: u32 = 0x4E4F534A;
    const CHUNK_TYPE_BIN: u32 = 0x004E4942;

    let total_len: u32 = (12                        // GLB header
        + 8 + json_bytes.len()                      // JSON chunk
        + 8 + bin_chunk.len()) as u32; // BIN  chunk

    let file = File::create(output_path)?;
    let mut w = BufWriter::new(file);

    // GLB header
    write_u32(&mut w, GLB_MAGIC)?;
    write_u32(&mut w, GLB_VERSION)?;
    write_u32(&mut w, total_len)?;

    // Chunk 0 — JSON
    write_u32(&mut w, json_bytes.len() as u32)?;
    write_u32(&mut w, CHUNK_TYPE_JSON)?;
    w.write_all(&json_bytes)?;

    // Chunk 1 — BIN
    write_u32(&mut w, bin_chunk.len() as u32)?;
    write_u32(&mut w, CHUNK_TYPE_BIN)?;
    w.write_all(&bin_chunk)?;

    println!(
        "Written: {}  ({} B, {} vertices, {} indices)",
        output_path,
        total_len,
        vertices.len(),
        indices.len()
    );
    Ok(())
}

fn main() -> std::io::Result<()> {
    let vertices = vec![
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
    ];
    let indices = vec![0u32, 1, 2];

    vec3_to_glb(&vertices, &indices, "output.glb")
}
