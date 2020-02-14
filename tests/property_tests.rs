use {
    quickcheck_macros::quickcheck,
    smu_huffman::{compress, decompress},
    std::collections::HashSet,
};

#[quickcheck]
fn roundtrip(bytes: Vec<u8>) -> bool {
    // Avoid the unhandled edge cases
    if bytes.iter().copied().collect::<HashSet<_>>().len() <= 1 {
        return true;
    }
    bytes == decompress(&compress(&bytes))
}
