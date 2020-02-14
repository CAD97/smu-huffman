use {
    quickcheck_macros::quickcheck,
    proptest::{prelude::*, collection::vec},
    smu_huffman::{compress, decompress},
};

#[quickcheck]
fn roundtrip(bytes: Vec<u8>) -> bool {
    bytes == decompress(&compress(&bytes))
}

proptest! {
    #[test]
    fn roundtrip_prop(bytes in vec(any::<u8>(), 0..10_000)) {
        prop_assert_eq!(&bytes, &decompress(&compress(&bytes)));
    }
}
