use std::{
    cell::RefCell,
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    error::Error,
    fmt::Display,
    rc::Rc,
};

use crate::bitstream::BitIndex;

pub const FIXED_CODES: [u8; 288] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
];

/// Struct representing each node of a prefix codetree. Used to both
/// represent branches and leaves. Where branches are the inner
/// nodes, and leaves are the outer nodes holding values.
///
/// # Fields
///
/// * 'leaf' - A boolean value which dictates whether the node is a leaf
///         and holds a value, or is a branch and holds other nodes.
/// * 'value' - A usize value representing the value held by the given leaf,
///         frequently, this represents symbols that are stored as the index
///         of arrays, so usize is used to ensure that an overflow will never
///         happen when converting index to value.
/// * 'significance' - A u32 value that represents how high up on the tree
///         a node should be. For compression this is almost always the frequency
///         a certain symbol occurs.
/// * 'address' - A u32 value that carries the address to the node on the tree,
///         this can be thought of as a list of lefts or rights represented by 0,
///         or 1, read from least significant to most significant bit.
/// * 'length' - A u8 representing the bit length of the address, a length
///         of 2 would mean take the 2 least significant bits the u32 address.
/// * 'left' - An option carrying a self-reference representing the node attached
///         to the left of current node.
/// * 'right' - An option carrying a self-reference representing the node attached
///         to the left of current node.
#[derive(Debug, Clone)]
pub struct Node {
    pub leaf: bool,
    pub value: usize,
    pub significance: u32,
    pub code: u32,
    pub length: u8,
    pub left: Option<Rc<RefCell<Node>>>,
    pub right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    /// Creates a new empty Node. Node implements default which calls this
    /// function, so using 'new_node = Node::default();' is identical to.
    /// 'new_node = Node::new();'
    ///
    /// # Returns
    ///
    /// A node with default values.
    pub fn new() -> Self {
        Self {
            leaf: false,
            value: 0,
            significance: 0,
            code: 0,
            length: 0,
            left: None,
            right: None,
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {:0length$b})",
            self.value,
            self.code,
            length = self.length as usize
        )
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.code.eq(&other.code)
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.code.cmp(&other.code)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct representing a prefix code tree.
///
/// # Fields
///
/// * 'root' - The root node to which all others are connected.
///  * 'leaves' - The nodes holding the final values.
pub struct PrefixTree {
    pub root: Rc<RefCell<Node>>,
    pub leaves: Vec<Rc<RefCell<Node>>>,
}

impl PrefixTree {
    /// Generates a prefix code tree from the given bit lengths.
    ///
    /// # Arguments
    ///
    /// * 'code_lengths' An array of u8 values representing the number of
    ///         bits in the code to represent a certain symbol, the symbol
    ///         is the index of the value. So, bit_lengths[1] would equal
    ///         the length of the code for 1.
    ///
    /// # Returns
    ///
    /// A new instance of PrefixTree built from the bit lengths provided.
    pub fn from_lengths(code_lengths: &[u8]) {
        // Define an array to hold the amount of times a code length appears.
        // The index is the code length, and the value at the index is the
        // number of occurances.
        let mut occurances = [0u32; 256];

        // Get the higest code length in the array.
        let max_length = *code_lengths.iter().max().unwrap_or(&0) as usize;

        // Iterates over code_lengths, taking occurances in as acc, and taking
        // the current iterated value as idx. Then, acc is dereferenced to
        // directly modify occurances, and it is indexed by idx (the code
        // length) before being incremented while preventing overflow by
        // saturating_add. acc is then returned, and the fold operation repeats
        // until all members of code_lengths have been iterated over.
        code_lengths.iter().fold(&mut occurances, |acc, &idx| {
            (*acc)[idx as usize] = (*acc)[idx as usize].saturating_add(1);
            acc
        });

        // Intialize next_code and code as zeroes.
        let mut next_code = vec![0; max_length + 1];
        let mut code = 0;
        occurances[0] = 0;

        // FIX: Still struggle to conceptualize this for some reason.
        for i in 1..=max_length {
            code = (code + occurances[i - 1]) << 1;
            next_code[i] = code;
        }

        let mut codes = vec![None; code_lengths.len()];

        for j in 0..code_lengths.len() {
            let len = code_lengths[j] as usize;
            if len != 0 {
                codes[j] = Some(next_code[len]);
                next_code[len] += 1;
            }
        }

        let mut leaves = Vec::new();
        let mut nodes_left = BinaryHeap::new();
        let mut nodes_right = BinaryHeap::new();

        for (sym, code) in codes.iter().enumerate() {
            if let Some(code) = code {
                let node = Node {
                    leaf: true,
                    value: sym,
                    significance: 0,
                    code: code.to_owned(),
                    length: code_lengths[sym],
                    left: None,
                    right: None,
                };

                leaves.push(Rc::new(RefCell::new(node.clone())));
                match code.bit_index(code_lengths[sym]) {
                    0 => nodes_left.push(Rc::new(RefCell::new(node.clone()))),
                    1 => nodes_right.push(Rc::new(RefCell::new(node.clone()))),
                    _ => {}
                }
            }
        }
    }
}
