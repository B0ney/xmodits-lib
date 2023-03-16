
struct GenericChunk {
    header: [u8; 4],
    chunk_size: i32,
    data: Box<[u8]>,
}

pub(crate) struct IFF;