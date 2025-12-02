//! NeoNet blockchain core skeleton
//! Fill in consensus, state, and networking modules.

pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: Vec<u8>,
    pub hash: String,
}

impl Block {
    pub fn genesis() -> Self {
        Block {
            index: 0,
            previous_hash: String::new(),
            timestamp: 0,
            data: vec![],
            hash: String::from("genesis"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn genesis_block() {
        let g = Block::genesis();
        assert_eq!(g.index, 0);
    }
}
