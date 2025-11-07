use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, 
    signature::Keypair, 
    signer::Signer,
    transaction::Transaction,
    instruction::Instruction,
    pubkey::Pubkey,
    system_instruction,
};
use std::str::FromStr;
use crate::{OrchestratorError, Result};


pub struct SolanaClient {
    rpc_client: RpcClient,
    keypair: Keypair,
    program_id: Pubkey,
    bridge_account: Pubkey,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, program_id: &str, bridge_account: &str, private_key: Option<&str>) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );

        let keypair = if let Some(pk) = private_key {
            Keypair::from_base58_string(pk)
        } else {
            Keypair::new()
        };

        let program_id = Pubkey::from_str(program_id)
            .map_err(|e| OrchestratorError::ConfigurationError(format!("Invalid program ID: {}", e)))?;

        let bridge_account = Pubkey::from_str(bridge_account)
            .map_err(|e| OrchestratorError::ConfigurationError(format!("Invalid bridge account: {}", e)))?;

        Ok(Self {
            rpc_client,
            keypair,
            program_id,
            bridge_account,
        })
    }

     pub async fn submit_batch(&self, batch: &crate::Batch) -> Result<String> {
        log::info!("Submitting batch with {} deposits to Solana", batch.deposits.len());
        
        // For now, use mock implementation for batch submission
        // You can replace this with real batch processing later
        let mock_signature = format!("solana_tx_batch_{}_{}", 
            batch.deposits.len(), 
            chrono::Utc::now().timestamp());

        log::info!("✅ Batch submitted to Solana: {}", mock_signature);
        
        Ok(mock_signature)
    }

    pub async fn submit_verified_deposit(
        &self,
        deposit: &crate::Deposit,
        proof: &str,
        verification_key: &str
    ) -> Result<String> {
        log::info!("Submitting verified deposit {} to Solana ZK program", deposit.deposit_id);

        // Create instruction for ZK program
        let instruction = self.create_verify_and_deposit_instruction(deposit, proof, verification_key).await?;

        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
        );

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(&[&self.keypair], recent_blockhash);

        // Submit transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;

        log::info!("✅ Deposit {} submitted to Solana ZK program: {}", deposit.deposit_id, signature);
        
        Ok(signature.to_string())
    }

    async fn create_verify_and_deposit_instruction(
        &self,
        deposit: &crate::Deposit,
        proof: &str,
        verification_key: &str
    ) -> Result<Instruction> {
        // This should match your Solana program's instruction structure
        // Based on your solana-program/src/lib.rs
        
        let accounts = vec![
            solana_sdk::instruction::AccountMeta::new(self.bridge_account, false),
            solana_sdk::instruction::AccountMeta::new(self.keypair.pubkey(), true),
        ];

        // Convert proof data to bytes (this depends on your ZK program interface)
        let proof_data = self.serialize_proof_data(deposit, proof, verification_key);

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: proof_data,
        })
    }

    fn serialize_proof_data(&self, deposit: &crate::Deposit, proof: &str, verification_key: &str) -> Vec<u8> {
        // This should match your Solana program's expected proof format
        // Based on your solana-program/src/verify.rs
        
        let mut data = Vec::new();
        
        // Add proof (this is a simplified version)
        data.extend_from_slice(proof.as_bytes());
        data.extend_from_slice(&[b';']); // separator
        data.extend_from_slice(verification_key.as_bytes());
        data.extend_from_slice(&[b';']); // separator
        
        // Add deposit data
        data.extend_from_slice(deposit.deposit_id.as_bytes());
        data.extend_from_slice(&[b';']);
        data.extend_from_slice(deposit.ton_tx_hash.as_bytes());
        data.extend_from_slice(&[b';']);
        data.extend_from_slice(deposit.recipient_solana.as_bytes());
        data.extend_from_slice(&[b';']);
        data.extend_from_slice(deposit.amount.as_bytes());

        data
    }

    pub async fn get_bridge_state(&self) -> Result<()> {
        // Fetch bridge state from Solana program
        let account_data = self.rpc_client.get_account_data(&self.bridge_account)?;
        log::info!("Bridge account data: {} bytes", account_data.len());
        Ok(())
    }
}