use anchor_lang::prelude::*;

#[account]
pub struct Escrow {
  pub is_initialized: bool,
  pub initializer_pubkey: Pubkey,
  pub tmp_token_account_pubkey: Pubkey,
  pub initializer_token_to_receive_account_pubkey: Pubkey,
  pub expected_amount: u64,
}
