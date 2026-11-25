//! The title's precompiled pipelines (device method 0x218) carry native GPU code but no shader
//! container. Each such program's first instructions hold its identity hash, and the same program
//! sits in a container of the shader pack (`shaders.rpf`, all 710 programs of a dump of the first
//! calls). This index maps that hash to the record of its container, so the GPU layer can make the
//! pipeline from the container as usual (`victory_gpu::pipeline::set_native_lookup`).
//!
//! The index is built once per pack (all cores, about a minute) and kept in the cache directory
//! as 24-byte entries: the 16-byte key, then the record's marker offset (u64).

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use victory_game::sgd5;
use victory_shader::{Container, FourCC, HwStage, native_programs};

/// The identity hash's bytes in a program's first 24: the literals of its first instructions.
pub(crate) fn key(code: &[u8]) -> Option<u128> {
    let c = code.get(..24)?;
    let mut k = [0u8; 16];
    k[..2].copy_from_slice(&c[0..2]);
    k[2..4].copy_from_slice(&c[4..6]);
    k[4..10].copy_from_slice(&c[8..14]);
    k[10..12].copy_from_slice(&c[16..18]);
    k[12..16].copy_from_slice(&c[20..24]);
    Some(u128::from_le_bytes(k))
}

struct Index {
    pack: PathBuf,
    records: HashMap<u128, u64>,
}

static INDEX: OnceLock<Index> = OnceLock::new();

/// Builds or loads the index of `pack` on a thread of its own and hands the GPU layer the lookup.
pub(crate) fn start(pack: PathBuf, cache: PathBuf) {
    if !pack.is_file() {
        tracing::warn!(pack = %pack.display(), "no shader pack: precompiled pipelines stay missing");
        return;
    }
    victory_gpu::pipeline::set_native_lookup(lookup);
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let records = load_or_build(&pack, &cache).unwrap_or_default();
        tracing::info!(programs = records.len(), ms = started.elapsed().as_millis() as u64, "shader pack index for precompiled pipelines");
        let _ = INDEX.set(Index { pack, records });
    });
}

fn load_or_build(pack: &Path, cache: &Path) -> std::io::Result<HashMap<u128, u64>> {
    let len = std::fs::metadata(pack)?.len();
    let file = cache.join(format!("native-pack-{len:x}.idx"));
    if let Ok(bytes) = std::fs::read(&file) {
        return Ok(bytes
            .as_chunks::<24>()
            .0
            .iter()
            .map(|e| (u128::from_le_bytes(e[..16].try_into().unwrap()), u64::from_le_bytes(e[16..].try_into().unwrap())))
            .collect());
    }
    let data = std::fs::read(pack)?;
    let records: Vec<sgd5::Record<'_>> = sgd5::records(&data).collect();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let parts: Vec<Vec<(u128, u64)>> = std::thread::scope(|s| {
        records
            .chunks(records.len().div_ceil(threads).max(1))
            .map(|chunk| s.spawn(move || chunk.iter().flat_map(keys_of).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });
    let map: HashMap<u128, u64> = parts.into_iter().flatten().collect();
    let mut out = Vec::with_capacity(map.len() * 24);
    for (k, o) in &map {
        out.extend_from_slice(&k.to_le_bytes());
        out.extend_from_slice(&o.to_le_bytes());
    }
    let _ = std::fs::write(&file, out);
    Ok(map)
}

/// The keys of a record's native programs, each with the record's offset.
fn keys_of(record: &sgd5::Record<'_>) -> Vec<(u128, u64)> {
    let Some(bytes) = record.unpack() else { return Vec::new() };
    let Ok(container) = Container::parse(&bytes) else { return Vec::new() };
    let Some(Ok(programs)) = container.part(FourCC::DXIL).map(|p| native_programs(p.data)) else { return Vec::new() };
    programs.iter().filter_map(|p| key(&p.code)).map(|k| (k, record.offset as u64)).collect()
}

/// The container whose native program starts with `code`, and that program's stage as the GPU
/// layer names it; waits for the index the first time.
fn lookup(code: &[u8]) -> Option<(Vec<u8>, &'static str)> {
    let key = key(code)?;
    let index = loop {
        if let Some(i) = INDEX.get() {
            break i;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let &offset = index.records.get(&key)?;
    // The record's name precedes its marker (at most 256 bytes) and its packed size is the u32
    // before it; read that window and find the record in it.
    let mut file = std::fs::File::open(&index.pack).ok()?;
    let start = offset.saturating_sub(300);
    file.seek(SeekFrom::Start(offset - 4)).ok()?;
    let mut n = [0u8; 4];
    file.read_exact(&mut n).ok()?;
    let end = offset + u64::from(u32::from_le_bytes(n));
    let mut window = vec![0u8; (end - start) as usize];
    file.seek(SeekFrom::Start(start)).ok()?;
    file.read_exact(&mut window).ok()?;
    let local = (offset - start) as usize;
    let container = sgd5::records(&window).find(|r| r.offset == local)?.unpack()?;
    let stage = {
        let parsed = Container::parse(&container).ok()?;
        let programs = native_programs(parsed.part(FourCC::DXIL)?.data).ok()?;
        match programs.iter().find(|p| self::key(&p.code) == Some(key))?.stage {
            HwStage::Vs | HwStage::Es | HwStage::Ls => "vertex",
            HwStage::Ps => "pixel",
            HwStage::Gs => "geometry",
            HwStage::Hs => "hull",
            HwStage::Ms => "mesh",
            HwStage::Cs => "compute",
        }
    };
    Some((container, stage))
}

#[cfg(test)]
mod tests {
    #[test]
    fn key_takes_the_identity_bytes_of_the_prologue() {
        let code: Vec<u8> = (0u8..32).collect();
        let k = super::key(&code).unwrap().to_le_bytes();
        assert_eq!(k, [0, 1, 4, 5, 8, 9, 10, 11, 12, 13, 16, 17, 20, 21, 22, 23]);
        assert!(super::key(&code[..20]).is_none());
    }
}
