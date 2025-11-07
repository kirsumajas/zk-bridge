// state.rs
use anchor_lang::prelude::*;

#[account]
pub struct LcState {
    pub admin: Pubkey,
    pub last_verified_slot: u64,
    pub vk_id: u32,
    pub ton_state_root: [u8; 32],  // ADD: Current TON state root
    pub relayer: Pubkey,           // ADD: Authorized relayer for state updates
}

impl LcState {
    pub const SEED: &'static [u8] = b"lc_state";
    pub const SIZE: usize = 32 + 8 + 4 + 32 + 32 + 8; // Updated size
}

#[account]
pub struct VerifyingKey {
    pub vk_id: u32,
    pub data: Vec<u8>,
}

impl VerifyingKey {
    pub const SEED: &'static [u8] = b"vk";
}

// Event state to prevent double-spending
#[account]
pub struct EventState {
    pub consumed: bool,
    pub event_id: [u8; 32],
    pub recipient: Pubkey,
    pub amount: u64,
    pub ton_tx_hash: [u8; 32],     // ADD: TON transaction hash
    pub ton_sender: [u8; 32],      // ADD: TON sender address
}

impl EventState {
    pub const SEED: &'static [u8] = b"event";
    pub const SIZE: usize = 1 + 32 + 32 + 8 + 32 + 32 + 8; // Updated size
}

// Enhanced public inputs for TON event verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EventPublicInputs {
    pub domain: [u8; 32],
    pub anchor_root: [u8; 32],
    pub event_id: [u8; 32],
    pub token_id: [u8; 32],
    pub amount_in_ton: u64,
    pub recipient_solana: Pubkey,
    pub fee_bps: u16,
    pub vk_version: u32,
    pub ton_tx_hash: [u8; 32],     // ADD: TON transaction hash
    pub ton_sender: [u8; 32],      // ADD: TON sender address
    pub nullifier: [u8; 32],       // ADD: Double-spend protection
}