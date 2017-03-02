// Copyright 2017 Brian Langenberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/* This is the braindead version of Huffman tree parsing
 * which gives each bit its own node in the tree for
 * traversing one-by-one.
 * It'll need to be optimized in order to be viable for reading
 * real files.*/

use std::fmt;
use std::collections::BTreeMap;

pub enum ReadHuffmanTree<T: Copy> {
    Leaf(T),
    Tree(Box<[ReadHuffmanTree<T>;2]>)
}

enum WipHuffmanTree<T: Copy> {
    Empty,
    Leaf(T),
    Tree((Box<WipHuffmanTree<T>>, Box<WipHuffmanTree<T>>))
}

impl<T: Copy> WipHuffmanTree<T> {
    fn new_empty() -> WipHuffmanTree<T> {
        WipHuffmanTree::Empty
    }

    fn new_leaf(value: T) -> WipHuffmanTree<T> {
        WipHuffmanTree::Leaf(value)
    }

    fn new_tree() -> WipHuffmanTree<T> {
        WipHuffmanTree::Tree((Box::new(Self::new_empty()),
                              Box::new(Self::new_empty())))
    }

    fn into_read_tree(self) -> Result<ReadHuffmanTree<T>,HuffmanTreeError> {
        match self {
            WipHuffmanTree::Empty => {
                Err(HuffmanTreeError::MissingLeaf)
            }
            WipHuffmanTree::Leaf(v) => {
                Ok(ReadHuffmanTree::Leaf(v))
            }
            WipHuffmanTree::Tree((l, r)) => {
                let new_tree = [l.into_read_tree()?, r.into_read_tree()?];
                Ok(ReadHuffmanTree::Tree(Box::new(new_tree)))
            }
        }
    }

    fn add(&mut self, bits: &[u8], value: T) -> Result<(),HuffmanTreeError> {
        match self {
            &mut WipHuffmanTree::Empty => {
                if bits.len() == 0 {
                    *self = WipHuffmanTree::new_leaf(value);
                    Ok(())
                } else {
                    *self = WipHuffmanTree::new_tree();
                    self.add(bits, value)
                }
            }
            &mut WipHuffmanTree::Leaf(_) => {
                Err(if bits.len() == 0 {
                    HuffmanTreeError::DuplicateLeaf
                } else {
                    HuffmanTreeError::OrphanedLeaf
                })
            }
            &mut WipHuffmanTree::Tree((ref mut l, ref mut r)) => {
                if bits.len() == 0 {
                    Err(HuffmanTreeError::DuplicateLeaf)
                } else {
                    match bits[0] {
                        0 => {l.add(&bits[1..], value)}
                        1 => {r.add(&bits[1..], value)}
                        _ => {Err(HuffmanTreeError::InvalidBit)}
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum HuffmanTreeError {
    InvalidBit,
    MissingLeaf,
    DuplicateLeaf,
    OrphanedLeaf,
    DuplicateValue
}

impl fmt::Display for HuffmanTreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HuffmanTreeError::InvalidBit => {
                write!(f, "invalid bit in specification")
            }
            HuffmanTreeError::MissingLeaf => {
                write!(f, "missing leaf node in specification")
            }
            HuffmanTreeError::DuplicateLeaf => {
                write!(f, "duplicate leaf node in specification")
            }
            HuffmanTreeError::OrphanedLeaf => {
                write!(f, "orphaned leaf node in specification")
            }
            HuffmanTreeError::DuplicateValue => {
                write!(f, "duplicate value in specification")
            }
        }
    }
}
/// Given a slice of bits/value pairs, compiles a Huffman tree for reading.
/// Bits must be 0 or 1 and are always consumed from the stream
/// from least-significant in the list to most signficant
/// (which makes them easier to read for humans).
/// Value can be anything that implements the Copy trait.
///
/// ## Example
/// ```
/// use bitstream_io::huffman::compile_read;
/// assert!(compile_read(&[(vec![0], 1i32),
///                        (vec![1, 0], 2i32),
///                        (vec![1, 1], 3i32)]).is_ok());
/// ```
pub fn compile_read<T: Copy>(values: &[(Vec<u8>, T)]) ->
    Result<ReadHuffmanTree<T>,HuffmanTreeError> {
    let mut tree = WipHuffmanTree::new_empty();

    for &(ref bits, ref value) in values {
        tree.add(bits.as_slice(), *value)?;
    }

    tree.into_read_tree()
}

pub type WriteHuffmanTree<T> = BTreeMap<T,Vec<u8>>;

pub fn compile_write<T: Ord + Copy>(values: &[(Vec<u8>, T)]) ->
    Result<WriteHuffmanTree<T>,HuffmanTreeError> {
    use std::collections::btree_map::Entry;

    let mut tree = BTreeMap::new();

    for &(ref bits, ref value) in values {
        match tree.entry(*value) {
            Entry::Vacant(entry) => {entry.insert(bits.clone());}
            Entry::Occupied(_) => {return Err(HuffmanTreeError::DuplicateValue)}
        }
    }
    Ok(tree)
}