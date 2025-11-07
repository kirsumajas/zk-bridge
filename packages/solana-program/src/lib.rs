// lib.rs
use anchor_lang::prelude::*;
mod state;
mod verify;
mod zk_verifier;
use state::*;
use zk_verifier::{ZkError, ZKProof};

declare_id!("8zcmz77ahioCSGX7QnmFL51a1A3qBY1nw5az7R11KF9o");

#[program]
pub mod zk_lc {
    use super::*;

    pub fn init_state(
        ctx: Context<InitState>, 
        vk_id: u32,
        initial_ton_root: [u8; 32],
        relayer: Pubkey,
    ) -> Result<()> {
        let s = &mut ctx.accounts.state;
        s.admin = ctx.accounts.payer.key();
        s.last_verified_slot = 0;
        s.vk_id = vk_id;
        s.ton_state_root = initial_ton_root;
        s.relayer = relayer;

        let vk = &mut ctx.accounts.verifying_key;
        vk.vk_id = vk_id;
        vk.data = vec![]; // Would load actual verification key
        
        Ok(())
    }

    pub fn update_ton_root(
        ctx: Context<UpdateTonRoot>,
        new_ton_root: [u8; 32],
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Only admin or relayer can update TON root
        require!(
            ctx.accounts.signer.key() == state.admin || 
            ctx.accounts.signer.key() == state.relayer,
            ZkError::UnauthorizedRelayer
        );
        
        state.ton_state_root = new_ton_root;
        
        msg!("TON state root updated to: {:?}", new_ton_root);
        Ok(())
    }

    pub fn verify_update(ctx: Context<VerifyUpdate>, new_slot: u64, proof: [u8; 32]) -> Result<()> {
        let s = &mut ctx.accounts.state;
        verify::verify_mock(&proof, s.last_verified_slot, new_slot)?;
        s.last_verified_slot = new_slot;
        Ok(())
    }

    // Enhanced TON Event Verification with ZK Proofs
    pub fn verify_ton_event(
        ctx: Context<VerifyTonEvent>,
        proof: ZKProof,  // Use structured proof instead of raw bytes
        public_inputs: EventPublicInputs,
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        
        // Verify the ZK proof
        zk_verifier::ZKVerifier::verify_ton_event_proof(
            &proof,
            &public_inputs,
            &state.ton_state_root,
            &ctx.accounts.verifying_key.data,
        )?;
        
        // Check if event was already consumed via nullifier
        let nullifier_account = &mut ctx.accounts.nullifier_account;
        require!(!nullifier_account.consumed, ZkError::EventAlreadyConsumed);
        
        // Mark nullifier as consumed
        nullifier_account.consumed = true;
        nullifier_account.nullifier = public_inputs.nullifier;
        nullifier_account.ton_tx_hash = public_inputs.ton_tx_hash;
        
        // Create event account
        let event_account = &mut ctx.accounts.event_account;
        event_account.consumed = true;
        event_account.event_id = public_inputs.event_id;
        event_account.recipient = public_inputs.recipient_solana;
        event_account.amount = public_inputs.amount_in_ton;
        event_account.ton_tx_hash = public_inputs.ton_tx_hash;
        event_account.ton_sender = public_inputs.ton_sender;

        // Emit event for indexers
        emit!(TonEventVerified {
            event_id: public_inputs.event_id,
            recipient: public_inputs.recipient_solana,
            amount: public_inputs.amount_in_ton,
            ton_tx_hash: public_inputs.ton_tx_hash,
            ton_sender: public_inputs.ton_sender,
        });

        msg!(
            "✅ TON event verified: {} TON to {}",
            public_inputs.amount_in_ton,
            public_inputs.recipient_solana
        );
        
        Ok(())
    }
}

// Event for indexing
#[event]
pub struct TonEventVerified {
    pub event_id: [u8; 32],
    pub recipient: Pubkey,
    pub amount: u64,
    pub ton_tx_hash: [u8; 32],
    pub ton_sender: [u8; 32],
}

#[derive(Accounts)]
pub struct InitState<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + LcState::SIZE,
        seeds = [LcState::SEED],
        bump
    )]
    pub state: Account<'info, LcState>,

    #[account(
        init,
        payer = payer,
        space = 8 + 4 + 4 + 1024,
        seeds = [VerifyingKey::SEED, &0u32.to_le_bytes()],
        bump
    )]
    pub verifying_key: Account<'info, VerifyingKey>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTonRoot<'info> {
    #[account(
        mut,
        seeds = [LcState::SEED],
        bump
    )]
    pub state: Account<'info, LcState>,
    
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyUpdate<'info> {
    #[account(
        mut,
        seeds = [LcState::SEED],
        bump
    )]
    pub state: Account<'info, LcState>,
}

// Enhanced TON event verification accounts - Remove Bumps derive
#[derive(Accounts)]
#[instruction(proof: ZKProof, public_inputs: EventPublicInputs)]
pub struct VerifyTonEvent<'info> {
    #[account(
        seeds = [LcState::SEED],
        bump
    )]
    pub state: Account<'info, LcState>,

    #[account(
        seeds = [VerifyingKey::SEED, &0u32.to_le_bytes()],
        bump
    )]
    pub verifying_key: Account<'info, VerifyingKey>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + EventState::SIZE,
        seeds = [EventState::SEED, &public_inputs.event_id],
        bump
    )]
    pub event_account: Account<'info, EventState>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + NullifierState::SIZE,
        seeds = [NullifierState::SEED, &public_inputs.nullifier],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierState>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Add nullifier state to prevent double spending
#[account]
pub struct NullifierState {
    pub consumed: bool,
    pub nullifier: [u8; 32],
    pub ton_tx_hash: [u8; 32],
}

impl NullifierState {
    pub const SEED: &'static [u8] = b"nullifier";
    pub const SIZE: usize = 1 + 32 + 32 + 8;
}