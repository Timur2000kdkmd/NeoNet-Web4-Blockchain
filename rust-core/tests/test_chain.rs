use std::sync::{Arc, Mutex};
use crate::Chain;
#[test]
fn test_chain_mine_and_validate() {
    let mut c = Chain::new();
    c.add_tx(crate::Tx{from: String::from("a"), to: String::from("b"), payload: String::from("p"), nonce: 0});
    let _ = c.mine_block("miner");
    assert!(c.validate());
}
