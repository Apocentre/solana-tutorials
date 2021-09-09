use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  program_error::ProgramError,
  msg,
  program_pack::{Pack, IsInitialized},
  pubkey::Pubkey,
  sysvar::{rent::Rent, Sysvar},
  program::invoke,
};

use crate::{
  instruction::EscrowInstruction,
  error::EscrowError,
  state::Escrow,
};

pub struct Processor;

impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;

    match instruction {
      EscrowInstruction::InitEscrow {amount} => {
        msg!("Instruction: Init Escrow");
        Self::process_init_escrow(accounts, amount, program_id)
      }
    }
  }

  fn process_init_escrow(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey
  ) -> ProgramResult {
    let account_info_iter =  &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let tmp_token_account = next_account_info(account_info_iter)?;
    let token_to_receive_account = next_account_info(account_info_iter)?;

    if *token_to_receive_account.owner != spl_token::id() {
      return Err(ProgramError::IncorrectProgramId);
    }

    let escrow_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
      return Err(EscrowError::NotRentExempt.into());
    }

    let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
    if escrow_info.is_initialized() {
      return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_info.is_initialized = true;
    escrow_info.initializer_pubkey = *initializer.key;
    escrow_info.tmp_token_account_pubkey = *tmp_token_account.key;
    escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
    escrow_info.expected_amount = amount;

    Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;


    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
    // transfer the ownership of the tmp token account to the newly create PDA
    let token_program = next_account_info(account_info_iter)?;
    let ownership_change_instruction = spl_token::instruction::set_authority(
      token_program.key,
      tmp_token_account.key,
      Some(&pda),
      spl_token::instruction::AuthorityType::AccountOwner,
      initializer.key,
      &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    invoke(
      &ownership_change_instruction,
      &[
          tmp_token_account.clone(),
          initializer.clone(),
          token_program.clone(),
      ],
    )?;

    Ok(())
  }
}
