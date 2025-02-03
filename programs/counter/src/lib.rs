use anchor_lang::prelude::*;
use solana_program::program::{invoke_signed};
use solana_program::system_instruction;
use anchor_spl::{
    token,
};
declare_id!("6YKvBAKt9BrWicEit9EW8EWknKawy4s1Mr2qTc1gnupW");

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

    pub fn token_transfer(ctx:Context<TransferSplToken>, amount: u64) -> Result<()>{
        let seeds = &[b"token_vault".as_ref(), &[ctx.bumps.token_vault]];
        let signer_seeds = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_account = token::Transfer{
            authority: ctx.accounts.signer.to_account_info(),
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.token_vault_ata.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);
        let _ = token::transfer(
            cpi_ctx,
            amount
        )?;
        Ok(())
    }
    
    pub fn token_withdraw(ctx: Context<WithDrawToken>, amount: u64) -> Result<()> {
        let counter = &ctx.accounts.counter;
        let seeds = &[b"counter".as_ref(), &[counter.bump]];
        let signer_seeds = &[&seeds[..]];
    
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_account = token::Transfer {
            authority: ctx.accounts.counter.to_account_info(),
            from: ctx.accounts.token_vault.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);
    
        token::transfer(cpi_ctx, amount)?;
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
    ///CHECK:
    #[account(
        init,
        seeds = [b"token_vault".as_ref()],
        bump,
        payer = user,
        space = 8 + 8 * 100,
    )]
    pub token_vault: AccountInfo<'info>,
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

#[derive(Accounts)]
#[instruction()]
pub struct TransferSplToken<'info> {
    #[account(mut)]
    pub mint_token:Account<'info, token::Mint>,
    #[account(mut)]
    pub user_token: Account<'info, token::TokenAccount>,
    ///CHECK:
    #[account(
        mut,
        seeds = [b"token_vault".as_ref()],
        bump
    )]
    pub token_vault : AccountInfo<'info>,
    #[account(mut)]
    pub token_vault_ata : Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program:Program<'info, System>,
    pub token_program:Program<'info, token::Token>,
}

#[derive(Accounts)]
#[instruction()]
pub struct WithDrawToken<'info> {
    #[account(mut)]
    pub mint_token:Account<'info, token::Mint>,
    #[account(mut)]
    pub user_token: Account<'info, token::TokenAccount>,
    #[account(
        mut,
        seeds = [b"token_vault".as_ref()],
        bump
    )]
    pub token_vault : Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"counter"], 
        bump,
    )]
    pub counter: Account<'info, Counter>,
    pub system_program:Program<'info, System>,
    pub token_program:Program<'info, token::Token>,

}


#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64, // 8 bytes
    pub bump: u8,   // 1 byte
}
