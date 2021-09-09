use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::{Pack, IsInitialized}, pubkey::Pubkey, sysvar::{rent::Rent, Sysvar}};
use spl_token::state::{Account as TokenAccount};
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
      },
      EscrowInstruction::Exchange {amount} => {
        msg!("Instruction: Exchange");
        Self::process_exchange(accounts, amount, program_id)
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


    // If we had't and Alice were to pass in a non-rent-exempt account, the account balance might 
    // go to zero before Bob takes the trade. With the account gone, Alice would have no way to 
    // recover her tokens.
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

  fn process_exchange(
    accounts: &[AccountInfo],
    amount_expected_by_taker: u64,
    program_id: &Pubkey
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let taker = next_account_info(account_info_iter)?;

    if !taker.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let takers_sending_token_account = next_account_info(account_info_iter)?;
    let takers_receiving_token_account = next_account_info(account_info_iter)?;
    let pda_temp_token_account = next_account_info(account_info_iter)?;
    let pda_temp_token_account_info = TokenAccount::unpack(&pda_temp_token_account.data.borrow())?;
    let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

    if amount_expected_by_taker != pda_temp_token_account_info.amount {
      return Err(EscrowError::ExpectedAmountMismatch.into());
    }

    let initializer_main_account = next_account_info(account_info_iter)?;
    let initializers_token_receive_account = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

    if escrow_info.tmp_token_account_pubkey != *pda_temp_token_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_pubkey != *initializer_main_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_token_to_receive_account_pubkey != *initializers_token_receive_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    let token_program = next_account_info(account_info_iter)?;

    let transfer_to_initializer_ix = spl_token::instruction::transfer(
      token_program.key,
      takers_sending_token_account.key,
      initializers_token_receive_account.key,
      taker.key,
      &[&taker.key],
      escrow_info.expected_amount
    )?;

    msg!("Calling the token program to transfer tokens to the escrow's initializer...");
    invoke(
      &transfer_to_initializer_ix,
      &[
        takers_sending_token_account.clone(),
        initializers_token_receive_account.clone(),
        taker.clone(),
        token_program.clone(),
      ]
    )?;

    // transfer the escrowed amount
    let pda_account = next_account_info(account_info_iter)?;
    let transfer_to_taker_ix = spl_token::instruction::transfer(
      token_program.key,
      pda_temp_token_account.key,
      takers_receiving_token_account.key,
      &pda,
      &[&pda],
      pda_temp_token_account_info.amount
    )?;

    msg!("Calling the token program to transfer tokens to the taker...");
    invoke_signed(
      &transfer_to_taker_ix,
      &[
        pda_temp_token_account.clone(),
        takers_receiving_token_account.clone(),
        pda_account.clone(),
        token_program.clone()
      ],
      &[&[&b"escrow"[..], &[bump_seed]]]
    )?;

    let close_pda_account_ix = spl_token::instruction::close_account(
      token_program.key,
      pda_temp_token_account.key,
      initializer_main_account.key,
      escrow_account.key,
      &[&pda]
    )?;

    msg!("Calling the token program to close pda's temp account...");
    invoke_signed(
      &close_pda_account_ix,
      &[
        pda_temp_token_account.clone(),
        initializer_main_account.clone(),
        pda_account.clone(),
        token_program.clone()
      ],
      &[&[&b"escrow"[..], &[bump_seed]]]
    )?;

    msg!("Closing the escrow account...");

    // transfer the lamports balance of the escrow account to the initializer main account
    // Even though the escrow program is not the owner of her account because we are 
    // crediting lamports to her account
    //
    // From the first sight it looks like the program can randomly increase the balance of any
    // account. However, the is some runtime checks that will assure that not invalid account 
    // modifications has happened i.e. the total balance of SOL of all accounts in the instruction
    // should remain the same
    **initializer_main_account.lamports.borrow_mut() = initializer_main_account.lamports()
      .checked_add(escrow_account.lamports())
      .ok_or(EscrowError::AmountOverflow)?;


    // Note we are debiting the escrow accounts sol balance and mutating its data
    // This is possible because the owner of the escrow account is this escrow program
    // and this gives it the right to do so
    **escrow_account.lamports.borrow_mut() = 0;
    // Why is this necessary if the account is purged from memory after the transaction anyway? 
    // It is because this instruction is not necessarily the final instruction in the transaction. 
    // Thus, a subsequent transaction may read or even revive the data completely by making the account rent-exempt again
    // Depending on your program, forgetting to clear the data field can have dangerous consequences.
    *escrow_account.data.borrow_mut() = &mut [];

    Ok(())
  }
}
