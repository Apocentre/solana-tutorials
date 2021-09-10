use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use crate::account::Escrow;

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
