use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
  system_program,
  program_pack::{IsInitialized},
  sysvar::{rent::Rent, Sysvar}
};
use anchor_spl::token;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod escrow_anchor {
  use super::*;

  pub fn init_escrow(ctx: Context<InitEscrow>, amount: u64) -> ProgramResult {
    let accounts = ctx.accounts;

    if !accounts.initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }
    if *accounts.token_to_receive_account.owner != token::ID {
      return Err(ProgramError::IncorrectProgramId);
    }
    // If we had't and Alice were to pass in a non-rent-exempt account, the account balance might 
    // go to zero before Bob takes the trade. With the account gone, Alice would have no way to 
    // recover her tokens.
    let rent  = &Rent::from_account_info(&accounts.rent)?;
    let escrow_account = accounts.escrow_account.to_account_info();
    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
      return Err(EscrowError::NotRentExempt.into());
    }

    let escrow_info = &mut accounts.escrow_account;
    if escrow_info.is_initialized() {
      return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_info.is_initialized = true;
    escrow_info.initializer_pubkey = *accounts.initializer.key;
    escrow_info.tmp_token_account_pubkey = *accounts.tmp_token_account.key;
    escrow_info.initializer_token_to_receive_account_pubkey = *accounts.token_to_receive_account.key;
    escrow_info.expected_amount = amount;

    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], ctx.program_id);
    
    let token_program = accounts.token_program.clone();
    let cpi_accounts = token::SetAuthority {
      current_authority: accounts.initializer.clone(),
      account_or_mint: accounts.tmp_token_account.clone(),
    };

    let cpi_ctx = CpiContext::new(token_program, cpi_accounts);
    
    msg!("Calling the token program to transfer token account ownership...");

    token::set_authority(
      cpi_ctx,
      spl_token::instruction::AuthorityType::AccountOwner,
      Some(pda)
    )?;

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
  // #[account(address = system_program::ID)]
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

impl IsInitialized for Escrow {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}


#[error]
pub enum EscrowError {
  #[msg("Invalid Instruction")]
  InvalidInstruction,

  #[msg("Not Rent Exempt")]
  NotRentExempt,

  #[msg("Expected Amount Mismatch")]
  ExpectedAmountMismatch,

  #[msg("Amount Overflow")]
  AmountOverflow,
}
