// verify.rs
use anchor_lang::prelude::*;
use crate::zk_verifier::ZkError;

pub fn verify_mock(proof: &[u8; 32], old_slot: u64, new_slot: u64) -> Result<()> {
    require!(new_slot > old_slot, ZkError::SlotGoesBackwards);
    require!(proof.iter().all(|b| *b == 0xAA), ZkError::BadProof);
    Ok(())
}