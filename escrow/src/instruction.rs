use std::convert::TryInto;
use solana_program::program_error::ProgramError;

use crate::error::EscrowError::InvalidInstruction;

pub enum EscrowInstruction {
  /// Start the trade by creating and populating and escrow account and transferring ownership of the
  /// given temp token account to the PDA
  ///
  /// Accounts expected:
  ///
  /// 0. `[signer]` the account of the person initializing the escrow
  /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by 
  ///     the initializer
  /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
  /// 3. `[writable]` The escrow account, it will hold all necessary information about the trade
  /// 4. `[]` The rent sysvar
  /// 5. `[]` The token program
  InitEscrow {
    amount: u64
  },

  /// Accepts aa trade
  ///
  /// Accounts expected:
  ///
  /// 0. `[signer]` The account of the person taking the trade
  /// 1. `[writable]` The taker's token account for the token they send
  /// 2. `[writable]` The taker's token account for the token they will receive
  /// 3. `[writable]` The PDA's token account from which the taker will receive the tokens
  /// 4. `[writable]` The initializer's main account to to send the rent fees after PDA and tmp token accounts are cleaned up
  /// 5. `[writable]` The initializer's token account that will receive the tokens
  /// 6. `[writable]` The escrow account holding the escrow info
  /// 7. `[]` The token program
  /// 8. `[]` The PDA account
  Exchange {
    // the amount the taker expects to be paid in the other token
    amount: u64
  }
}

impl EscrowInstruction {
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

    Ok(match tag {
      0 => Self::InitEscrow {
        amount: Self::unpack_amount(rest)?,
      },
      1 => Self::Exchange {
        amount: Self::unpack_amount(rest)?,
      },
      _ => return Err(InvalidInstruction.into()),
    })
  }

  pub fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
    let amount = input
      .get(..8)
      .and_then(|slice| slice.try_into().ok())
      .map(u64::from_le_bytes)
      .ok_or(InvalidInstruction)?;
    
    Ok(amount)
  }
}
