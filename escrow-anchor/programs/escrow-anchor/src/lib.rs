use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod escrow_anchor {
    use super::*;
    pub fn init_escrow(ctx: Context<instruction::InitEscrow>) -> ProgramResult {
      Ok(())
    }
}
