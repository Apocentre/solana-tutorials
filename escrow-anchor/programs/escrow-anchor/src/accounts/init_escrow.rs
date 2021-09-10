use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;


#[derive(Accounts)]
pub struct InitEscrow<'info> {
  #[account(signer)]
  pub initializer: AccountInfo<'info>,
  #[account(mut)]
  pub tmp_token_account: AccountInfo<'info>,
  pub token_to_receive_account: AccountInfo<'info>,
  #[account(mut)]
  pub escrow_account: AccountInfo<'info>,
  pub rent: AccountInfo<'info>,
  pub token_program: AccountInfo<'info>,
}
