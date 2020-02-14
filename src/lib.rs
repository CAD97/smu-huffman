//! Reference: <https://www.smu.edu/-/media/Site/guildhallOLD/Documents/Huffman_Exercise.pdf>

use {
    arr_macro::arr,
    bitvec::{prelude::*, slice::Iter as BitSliceIter},
    std::{
        cmp,
        convert::TryInto,
        fmt,
        ops::{Index, IndexMut},
        u8,
    },
};

pub fn compress(bytes: &[u8]) -> Vec<u8> {
    assert!(!bytes.is_empty());

    // 2. Loop through all bytes of the file, counting the frequency of
    // occurrence of each byte. You might want to print out the resulting
    // table of frequencies to get an impression of what you're doing.
    let byte_frequency = ByteFrequency::of(bytes);
    log::trace!("Byte frequency: {:?}", byte_frequency);

    // 3. Next, to assign a bit sequence to each character. The easiest way to
    // guarantee they are unique and optimal is by constructing a binary tree.

    // Create a leaf node out of each byte and put them in a list of nodes.
    let mut nodes: Vec<HuffmanCodingTree> = byte_frequency
        .iter()
        .filter(|&(_, frequency)| frequency > 0)
        .map(|(byte, frequency)| HuffmanCodingTree::Leaf { byte, frequency })
        .collect();
    // Keep it sorted as a sort of poor man's priority queue.
    nodes.sort_unstable_by_key(|node| cmp::Reverse(node.frequency()));
    assert!(nodes.len() > 1);

    // Repeat until the list contains just 1 node, the root of the tree.
    while nodes.len() > 1 {
        // Find in your list of nodes the two nodes that have the lowest frequency,
        // and take them out of the list. (The list is sorted high-to-low, above.)
        let left = nodes.pop().unwrap(); // NB: Vec::pop is the back element
        let right = nodes.pop().unwrap();

        // Create a new node (and insert it into list) and make these two nodes its children.
        // Make the frequency of this new node the sum of its children.
        let node = HuffmanCodingTree::Node {
            frequency: left.frequency() + right.frequency(),
            left: Box::new(left),
            right: Box::new(right),
        };
        let sorted_idx = nodes
            .binary_search_by_key(&cmp::Reverse(node.frequency()), |node| {
                cmp::Reverse(node.frequency())
            })
            .unwrap_or_else(|x| x);
        nodes.insert(sorted_idx, node);
    }

    let tree = nodes.pop().unwrap();
    assert!(nodes.is_empty());
    log::trace!("Huffman tree: {:?}", tree);

    // 4. Once you have the tree, recurse through the tree in depth-first
    // order, and collect the list of bits as a path from the root
    // (where 0 means left branch, and 1 means right).
    // Whenever you arrive at a leaf, the list is the set of bits for that
    // value. Store the bits. Note that the number of bits is variable;
    // it can easily be more than 8 for the less frequent values.
    let coding = HuffmanCoding::of(&tree);

    let mut bits: BitVec<Local, u8> = BitVec::new();

    // 6. To be able to decompress the file, you'll also need to store the tree
    // in the file. The easiest way to write a tree is to write it depth-first,
    // post-order, that way you can reconstruct it using a stack.
    // NB: We write the tree first because it is needed to decode the rest.
    tree.write_tree(&mut bits);

    // 5. Now loop through all the bytes in the file again. For each byte,
    // write out the corresponding bits to a file. To write bits to a file,
    // you'll need to accumulate them in bytes at a time...
    // this requires the use of bit operators.
    // NB: The bitvec library is OP
    for &byte in bytes {
        coding.push_byte(byte, &mut bits);
    }

    // To handle leftover trailing bits, we just put a u32 at the end
    // saying how many bits we actually want to decode from.
    // We have to slice it into bytes manually.
    let bit_count: u32 = bits.len().try_into().unwrap();
    let byte_0 = (bit_count >> (0 * 8)) as u8;
    let byte_1 = (bit_count >> (1 * 8)) as u8;
    let byte_2 = (bit_count >> (2 * 8)) as u8;
    let byte_3 = (bit_count >> (3 * 8)) as u8;
    let mut vec = bits.into_vec();
    vec.extend_from_slice(&[byte_0, byte_1, byte_2, byte_3]);

    vec
}

pub fn decompress(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() > 4);

    // To handle leftover trailing bits, we just put a u32 at the end
    // saying how many bits we actually want to decode from.
    // We have to reconstruct it from bytes manually.
    let bit_count: u32 = if let [byte_0, byte_1, byte_2, byte_3] = bytes[bytes.len() - 4..] {
        0 | (byte_0 as u32) << (0 * 8)
            | (byte_1 as u32) << (1 * 8)
            | (byte_2 as u32) << (2 * 8)
            | (byte_3 as u32) << (3 * 8)
    } else {
        unreachable!()
    };
    let mut bits = BitSlice::<Local, _>::from_slice(bytes)[..bit_count as usize].iter();

    // 1. Reconstruct the tree from the file. If you read a leaf,
    // put it on a stack, if you read a node, take 2 children from
    // the stack and put the node back on. This should leave you
    // just the root on the stack when you are done.
    let tree = HuffmanCodingTree::read_tree(&mut bits);

    // 2. To decompress, read bits from the remainder of the file,
    // and use each bit to walk down the tree (0 means go left,
    // 1 means go right). Whenever you come to a leaf, write the value of
    // that leaf to the output file, and start again at the root of the tree.
    let mut out = Vec::new();
    while !bits.as_bitslice().is_empty() {
        out.push(tree.pull_byte(&mut bits));
    }

    out
}

struct ByteFrequency {
    bytes: [usize; u8::MAX as usize + 1],
}

impl Index<u8> for ByteFrequency {
    type Output = usize;

    fn index(&self, index: u8) -> &usize {
        &self.bytes[index as usize]
    }
}

impl IndexMut<u8> for ByteFrequency {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.bytes[index as usize]
    }
}

impl Default for ByteFrequency {
    fn default() -> Self {
        Self { bytes: [0; 256] }
    }
}

// Cannot be derived because array impls stop at [_; 32]
impl fmt::Debug for ByteFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg_map = f.debug_map();
        for (byte, &count) in self.bytes.iter().enumerate() {
            dbg_map.entry(&byte, &count);
        }
        dbg_map.finish()
    }
}

impl ByteFrequency {
    fn of(bytes: &[u8]) -> Self {
        let mut this = ByteFrequency::default();
        for &byte in bytes {
            this[byte] += 1;
        }
        this
    }

    fn iter(&self) -> impl Iterator<Item = (u8, usize)> + '_ {
        self.bytes
            .iter()
            .enumerate()
            .map(|(byte, &frequency)| (byte.try_into().unwrap(), frequency))
    }
}

#[derive(Debug)]
enum HuffmanCodingTree {
    Leaf {
        byte: u8,
        frequency: usize,
    },
    Node {
        left: Box<HuffmanCodingTree>,
        right: Box<HuffmanCodingTree>,
        frequency: usize,
    },
}

impl HuffmanCodingTree {
    fn frequency(&self) -> usize {
        match *self {
            HuffmanCodingTree::Leaf { frequency, .. }
            | HuffmanCodingTree::Node { frequency, .. } => frequency,
        }
    }

    fn pull_byte<T: BitStore>(&self, bits: &mut BitSliceIter<Local, T>) -> u8 {
        match self {
            &HuffmanCodingTree::Leaf { byte, .. } => byte,
            HuffmanCodingTree::Node { left, right, .. } => match bits.next() {
                Some(false) => left.pull_byte(bits),
                Some(true) => right.pull_byte(bits),
                None => panic!("Huffman decode ran out of bits"),
            },
        }
    }

    fn write_tree<T: BitStore>(&self, bits: &mut BitVec<Local, T>) {
        match self {
            &HuffmanCodingTree::Leaf { byte, .. } => {
                bits.push(true);
                bits.extend_from_slice(byte.bits::<Local>());
            }
            HuffmanCodingTree::Node { left, right, .. } => {
                bits.push(false);
                left.write_tree(bits);
                right.write_tree(bits);
            }
        }
    }

    fn read_tree<T: BitStore>(bits: &mut BitSliceIter<Local, T>) -> Self {
        match bits.next() {
            Some(true) => {
                let byte: u8 = bits.as_bitslice()[..8].load();
                bits.take(8).for_each(drop);
                HuffmanCodingTree::Leaf { byte, frequency: 0 }
            }
            Some(false) => {
                let left = Self::read_tree(bits);
                let right = Self::read_tree(bits);
                HuffmanCodingTree::Node {
                    left: Box::new(left),
                    right: Box::new(right),
                    frequency: 0,
                }
            }
            None => panic!("Huffman decode ran out of bits"),
        }
    }
}

struct HuffmanCoding {
    codings: [Option<BitBox>; u8::MAX as usize + 1],
}

impl Index<u8> for HuffmanCoding {
    type Output = BitSlice;

    fn index(&self, index: u8) -> &BitSlice {
        self.codings[index as usize].as_ref().unwrap()
    }
}

// Cannot be derived because array impls stop at [_; 32]
impl fmt::Debug for HuffmanCoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg_map = f.debug_map();
        for (byte, count) in self.codings.iter().enumerate() {
            dbg_map.entry(&byte, &count);
        }
        dbg_map.finish()
    }
}

impl HuffmanCoding {
    fn of(tree: &HuffmanCodingTree) -> Self {
        let mut this = HuffmanCoding {
            // uuuuggggghhhh (arrays of size > 32 do not yet impl Default)
            codings: arr![None; 256],
        };
        let mut path = BitVec::new();
        this.apply(tree, &mut path);
        this
    }

    fn apply(&mut self, tree: &HuffmanCodingTree, path: &mut BitVec) {
        // 4. Once you have the tree, recurse through the tree in depth-first
        // order, and collect the list of bits as a path from the root
        // (where 0 means left branch, and 1 means right).
        // Whenever you arrive at a leaf, the list is the set of bits for that
        // value. Store the bits. Note that the number of bits is variable;
        // it can easily be more than 8 for the less frequent values.

        match tree {
            &HuffmanCodingTree::Leaf { byte, .. } => {
                let coding = &mut self.codings[byte as usize];
                assert!(coding.is_none());
                *coding = Some(path.clone().into_boxed_bitslice());
            }
            HuffmanCodingTree::Node { left, right, .. } => {
                path.push(false);
                self.apply(left, path);
                path.pop();
                path.push(true);
                self.apply(right, path);
                path.pop();
            }
        }
    }

    fn push_byte<T: BitStore>(&self, byte: u8, bits: &mut BitVec<Local, T>) {
        bits.extend_from_slice(&self[byte])
    }
}
