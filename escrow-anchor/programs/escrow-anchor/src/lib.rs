use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod escrow_anchor {
    use super::*;
    pub fn init_escrow(ctx: Context<InitEscrow>) -> ProgramResult {
      Ok(())
    }
}

#[derive(Accounts)]
pub struct InitEscrow<'info> {
  // 8 byte discriminator followed by 1 + 32 + 32 + 32 + 8
  // check here https://docs.solana.com/developing/on-chain-programs/overview
  #[account(init, payer = initializer, space = 8 + 105)]
  pub escrow_account: Account<'info, Escrow>,
  #[account(signer)]
  pub initializer: AccountInfo<'info>,
  #[account(mut)]
  pub tmp_token_account: AccountInfo<'info>,
  pub token_to_receive_account: AccountInfo<'info>,
  pub rent: AccountInfo<'info>,
  pub token_program: AccountInfo<'info>,

  // system_program, which is required by the runtime for creating the account
  pub system_program: AccountInfo<'info>,
}

#[account]
pub struct Escrow {
  pub is_initialized: bool,
  pub initializer_pubkey: Pubkey,
  pub tmp_token_account_pubkey: Pubkey,
  pub initializer_token_to_receive_account_pubkey: Pubkey,
  pub expected_amount: u64,
}
