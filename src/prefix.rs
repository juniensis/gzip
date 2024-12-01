use std::{cmp::Ordering, fmt, fmt::Display};

/// Code lengths from section 3.2.6 of RFC 1951.
pub const FIXED_CODE_LENGTHS: [u8; 288] = [
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

/// Length codes from section 3.2.5 of RFC 1951.
pub const LENGTH_CODES: [usize; 29] = [
    257, 258, 259, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 270, 271, 272, 273, 274, 275,
    276, 277, 278, 279, 280, 281, 282, 283, 284, 285,
];

/// The number of extra bits each length code has.
pub const LENGTH_EXTRA_BITS: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

/// The base length value of each length code.
pub const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];

/// Distance codes from section 3.2.5 of RFC 1951.
pub const DISTANCE_CODES: [usize; 30] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29,
];

/// The number of extra bits each distance code has.
pub const DISTANCE_EXTRA_BITS: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// The base value of each distance code.
pub const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];

/// A struct for representing codes of differing bit lengths, codes are stored
/// little endian, meant to be read from most significant bit to least
/// significant bit.
///
/// # Fields
///
/// * 'buffer' - A u32 acting as a bit buffer.
/// * 'length' - A u8 specifying how many of the bits in the buffer are actually
///         a part of the code. A length of 2 would mean that the 2 least
///         significant bits hold the code.
///
/// # Methods
///
/// * 'new' - Generates a new empty Code.
/// * 'from' - Accepts a buffer and a length and creates a Code struct from
///         those given values.
/// * 'push' - Accepts a buffer and a length and pushes length bits of value
///         into the bit buffer.
/// * 'push_bit' - Accepts a single u8 which is normalized to represent either
///         a 0 or 1, and pushes it to the buffer.
///
/// # Examples
///
/// '''
/// let new = Code::new();
/// let from = Code::from(0b1011, 4);
///
/// new_code.push_bit(1);
/// new_code.push(0b011, 3);
///
/// // Both codes now have a length of 4, and the u32 value:
/// // 0b0000_0000_0000_0000_0000_0000_0000_1011
/// assert_eq!(new.code, from.code);
/// '''
#[derive(Clone, Debug)]
pub struct Code {
    pub buffer: u32,
    pub length: u8,
    index: u8,
}

impl Code {
    /// Constructs a new, empty instance of Code.
    ///
    /// # Returns
    ///
    /// A Code struct with zeroes for all fields.
    pub fn new() -> Self {
        Self {
            buffer: 0,
            length: 0,
            index: 0,
        }
    }
    /// Constructs an instance of Code with a given code and length.
    ///
    /// # Arguments
    ///
    /// * 'code' - The u32 value containing the binary code.
    /// * 'length' - A u8 representing the number of bits of code are a part
    ///         of the binary code.
    ///
    /// # Returns
    ///
    /// A Code struct with the given values.
    pub fn from(buffer: u32, length: u8) -> Self {
        Self {
            buffer,
            length,
            index: 0,
        }
    }
    /// Accepts a length and a u32 as a buffer, and pushes length bits of that
    /// buffer into self.code and increments self.length by the appropriate
    /// amount.
    ///
    /// # Arguments
    ///
    /// * 'buffer' - A u32 acting as a bit buffer containing the bits to push.
    /// * 'length' - The number of bits to push.
    pub fn push(&mut self, buffer: u32, length: u8) {
        self.buffer = (self.buffer << length) | buffer;
        self.length += length;
    }
    /// Accepts either a 0 or 1 and pushes that bit to self. If a non-binary
    /// value is entered it will correct it to a 1 instead of raising an error.
    ///
    /// # Arguments
    ///
    /// * 'bit' - A u8 representing the bit to push.
    pub fn push_bit(&mut self, bit: u8) {
        let normalized_bit: u32 = match bit {
            0 => 0,
            1 => 1,
            _ => {
                eprintln!("Warning: Non-binary value passed to push, value corrected to a 1.");
                1
            }
        };

        self.buffer = (self.buffer << 1) | normalized_bit;
        self.length += 1;
    }
}

impl Default for Code {
    fn default() -> Self {
        Code::new()
    }
}

impl Iterator for Code {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            self.index += 1;
            Some((self.buffer >> (self.length - self.index) & 1) as u8)
        } else {
            None
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0length$b}",
            self.buffer,
            length = self.length as usize
        )
    }
}

/// Struct representing each node of a binary tree.
///
/// # Fields
///
/// * 'value' - An option containing a usize or None, None represents that
///         the node is a branch rather than a leaf, if a value is present,
///         the node should be on the edge of the tree.
/// * 'significance' - A u64 value used to sort the nodes on the tree. If
///         implementing a frequency based Huffman tree, significance can
///         be used to represent the frequency of each node. If, used to
///         generate prefix codes, significance represents the code.
/// * 'code' - An instance of the Code struct which contains a u32 bit buffer
///         containing the code, and a length representing what quantity of bits
///         in the buffer are part of the code.
/// * 'left' - An option holding a Box reference to the child node attached to
///         the left.
/// * 'right' - An option holding a Box reference to the child node attached to
///         the right.
#[derive(Debug, Clone)]
pub struct Node {
    pub value: Option<usize>,
    pub significance: u64,
    pub code: Code,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl Node {
    /// Creates a new empty Node.
    ///
    /// # Returns
    ///
    /// A node with default values.
    pub fn new() -> Self {
        Self {
            value: None,
            significance: 0,
            code: Code::new(),
            left: None,
            right: None,
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", self.value, self.code)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.significance.eq(&other.significance)
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.significance
            .cmp(&other.significance)
            .then_with(|| self.code.length.cmp(&other.code.length))
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

/// A binary tree containing prefix codes.
///
/// # Fields
///
/// * 'root' - The root node to which all others are connected.
/// * 'leaves' - A vector containing all the leaf nodes/nodes with values.
/// * 'current' - The most recent node to be traversed.
#[derive(Debug)]
pub struct PrefixTree {
    pub root: Node,
    pub current: Box<Node>,
}

impl PrefixTree {
    /// Creates a new empty PrefixTree.
    ///
    /// # Returns
    ///
    /// A PrefixTree with default values.
    pub fn new() -> Self {
        Self {
            root: Node::new(),
            current: Box::new(Node::new()),
        }
    }
    /// Accepts a code as input and then creates the branches required to reach
    /// the new node, and then populates the value specified.
    ///
    /// # Arguments
    ///
    /// * 'code' - A Code struct containing the address of the node to be
    ///         added to the tree.
    /// * 'value' - A usize value containing the value to be stored at the
    ///         new node.
    ///
    /// # Examples
    ///
    /// '''
    /// let tree = PrefixTree::new();
    ///
    /// let new_code = Code::from(0b011, 3);
    ///
    /// tree.insert_code(new_code, 255);
    ///
    /// let mut value = 0;
    /// for bit in code {
    ///     if let Some(v) = tree.walk(bit) {
    ///         value = v;
    ///     }
    /// }
    ///
    /// assert_eq!(value, 255);
    /// '''
    pub fn insert_code(&mut self, code: Code, value: usize) {
        let mut current = &mut self.root;
        let mut current_code = Code::new();
        for bit in code.clone() {
            match bit {
                0 => {
                    if current.left.is_none() {
                        current.left = Some(Box::new(Node::new()));
                    }
                    current = current.left.as_mut().unwrap();
                    current_code.push_bit(bit);
                    current.code = current_code.clone();
                }
                1 => {
                    if current.right.is_none() {
                        current.right = Some(Box::new(Node::new()));
                    }
                    current = current.right.as_mut().unwrap();
                    current_code.push_bit(bit);
                    current.code = current_code.clone();
                }
                _ => {}
            }
        }
        current.value = Some(value);
        current.code = code;
        self.current = Box::new(self.root.clone());
    }
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
    pub fn from_lengths(code_lengths: &[u8]) -> Self {
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

        let mut tree = PrefixTree::new();

        for (index, code) in codes.iter().enumerate() {
            if let Some(c) = code {
                let code_struct = Code::from(c.to_owned(), code_lengths[index]);
                tree.insert_code(code_struct, index);
            }
        }

        tree
    }
    /// Accepts a u8 representing a binary value and walks that direction on
    /// the tree. Will panic if a non-binary value is given.
    ///
    /// # Arguments
    ///
    /// * 'direction' - A u8 containing which direction on the tree to step.
    ///         Can be either 0 or 1.
    ///
    /// # Returns
    ///
    /// If the node is a branch and does not hold a value, None will be
    /// returned, otherwise, the value at that leaf will be returned.
    ///
    /// # Examples
    ///
    /// '''
    /// let mut tree = PrefixTree::new();
    ///
    /// tree.insert_code(Code::from(0b111, 3), 255);
    ///
    /// assert_eq!(tree.walk(1), None);
    /// assert_eq!(tree.walk(1), None);
    /// assert_eq!(tree.walk(1), Some(255));
    /// '''
    pub fn walk(&mut self, direction: u8) -> Option<usize> {
        assert!(direction < 2);

        match direction {
            0 => {
                if let Some(v) = self.current.left.clone() {
                    self.current = v.clone();
                    if let Some(value) = v.value {
                        self.current = Box::new(self.root.clone());
                        return Some(value);
                    } else {
                        return None;
                    }
                }
            }
            1 => {
                if let Some(v) = self.current.right.clone() {
                    self.current = v.clone();
                    if let Some(value) = v.value {
                        self.current = Box::new(self.root.clone());
                        return Some(value);
                    } else {
                        return None;
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl Default for PrefixTree {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PrefixTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_node(
            node: &Option<Box<Node>>,
            prefix: String,
            is_right: bool,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            if let Some(node) = node {
                writeln!(
                    f,
                    "{}{}({}{})",
                    prefix,
                    if is_right { "├── " } else { "└── " },
                    node.code,
                    if let Some(value) = node.value {
                        format!(": {}", value)
                    } else {
                        String::new()
                    }
                )?;
                let new_prefix = format!("{}{}", prefix, if is_right { "│   " } else { "    " });
                format_node(&node.right, new_prefix.clone(), true, f)?;
                format_node(&node.left, new_prefix, false, f)?;
            }
            Ok(())
        }

        writeln!(f, "{}", self.root)?;
        format_node(&self.root.right, String::new(), true, f)?;
        format_node(&self.root.left, String::new(), false, f)
    }
}
