use anchor_lang::prelude::*;
use solana_program::program::{invoke_signed};
use solana_program::system_instruction;

declare_id!("7bYcSLDB2T9XhPAysP9Xi7cmMpXbDSNa6EA79EPezDLU");

#[program]
pub mod counter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.bump = ctx.bumps.counter; // store bump seed in `Counter` account
        msg!("Counter account created! Current count: {}", counter.count);
        msg!("Counter bump: {}", counter.bump);
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;

        msg!("Previous counter: {}", counter.count);
        counter.count = counter.count.checked_add(1).unwrap();
        msg!("Counter incremented! Current count: {}", counter.count);

        // Derive PDA signer
        let seeds = &[b"counter".as_ref(), &[counter.bump]];
        let signer_seeds = &[&seeds[..]];

        // Ensure the PDA signs for the transaction
        invoke_signed(
            &system_instruction::assign(
                ctx.accounts.counter.to_account_info().key,
                ctx.program_id,
            ),
            &[
                ctx.accounts.counter.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    // Create and initialize `Counter` account using a PDA as the address
    #[account(
        init,
        seeds = [b"counter"], 
        bump,                 
        payer = user,
        space = 8 + Counter::INIT_SPACE
    )]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        seeds = [b"counter"], 
        bump = counter.bump,
    )]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64, // 8 bytes
    pub bump: u8,   // 1 byte
}
