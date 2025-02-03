use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, TransferChecked};
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};

declare_id!("F7x2NhDkEwa8inv4awyc1MBir2icyth7QawvoPubitUw");

const ADMIN_WALLET: Pubkey = Pubkey::new_from_array([117, 97, 68, 51, 98, 66, 70, 104, 56, 67, 112, 70, 71, 87, 114, 70, 119, 51, 118, 69, 57, 70, 104, 53, 56, 115, 52, 70, 88, 97, 72, 66]);

#[program]
pub mod private_vesting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vesting = &ctx.accounts.vesting;
        msg!("Vesting account created! Current vesting: {}", vesting.start_time);
        Ok(())
    }

    pub fn set_vesting(
        ctx: Context<SetVesting>, 
        start_time: u64, 
        sale_duration: u64, 
        vesting_duration: u64, 
        amount: u64
    ) -> Result<()> {
        // Only admin can deposit tokens
        require!(
            ctx.accounts.user.key() == ADMIN_WALLET,
            ErrorCode::Unauthorized
        );

        // Check if there's no active vesting or if previous vesting has ended
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            ctx.accounts.vesting.start_time == 0 || 
            current_time >= ctx.accounts.vesting.start_time + 
                (ctx.accounts.vesting.sale_duration + ctx.accounts.vesting.vesting_duration) as i64,
            ErrorCode::ActiveVestingExists
        );

        // Set vesting parameters
        ctx.accounts.vesting.start_time = Clock::get()?.unix_timestamp + start_time as i64;
        ctx.accounts.vesting.sale_duration = sale_duration;
        ctx.accounts.vesting.vesting_duration = vesting_duration;
        ctx.accounts.vesting.amount = amount;
        ctx.accounts.vesting.investor_claimed = 0;
        ctx.accounts.vesting.community_claimed = 0;

        // Transfer tokens from admin to vault
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.admin_token.clone().to_account_info(),
            to: ctx.accounts.token_vault.clone().to_account_info(),
            authority: ctx.accounts.user.clone().to_account_info(),
            mint: ctx.accounts.mint.clone().to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.clone().to_account_info(), 
            transfer_cpi_accounts
        );
        token_2022::transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;

        Ok(())
    }

    pub fn allow_claim(
        ctx: Context<AllowClaim>,
        amount: u64,
        is_investor: bool
    ) -> Result<()> {
        require!(
            ctx.accounts.from.key() == ADMIN_WALLET,
            ErrorCode::Unauthorized
        );

        if is_investor {
            require!(
                amount <= ctx.accounts.vesting.amount - ctx.accounts.vesting.investor_claimed,
                ErrorCode::AllocationAmountTooLarge
            );
            ctx.accounts.vesting.investor_claimed = ctx.accounts.vesting.investor_claimed.checked_add(amount).unwrap();
        } else {
            require!(
                amount <= ctx.accounts.vesting.amount - ctx.accounts.vesting.community_claimed,
                ErrorCode::AllocationAmountTooLarge
            );
            ctx.accounts.vesting.community_claimed = ctx.accounts.vesting.community_claimed.checked_add(amount).unwrap();
        }

        require!(
            ctx.accounts.vesting.start_time > Clock::get()?.unix_timestamp,
            ErrorCode::SaleNotStarted
        );

        require!(
            ctx.accounts.vesting.start_time + ctx.accounts.vesting.sale_duration as i64 > Clock::get()?.unix_timestamp,
            ErrorCode::SaleNotEnded
        );

        ctx.accounts.user_info.total_allocation = ctx.accounts.user_info.total_allocation.checked_add(amount).unwrap();
        ctx.accounts.user_info.claimed_amount = 0;
        ctx.accounts.user_info.is_investor = is_investor;
        Ok(())
    }

    pub fn claim_token(ctx: Context<ClaimToken>, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        // Ensure sale has ended
        require!(
            current_time >= ctx.accounts.vesting.start_time + ctx.accounts.vesting.sale_duration as i64,
            ErrorCode::SaleNotEnded
        );

        // Calculate initial 15% allocation
        let initial_allocation = ctx.accounts.user_info.total_allocation.checked_mul(15).unwrap().checked_div(100).unwrap();
        
        // Calculate vesting amount
        let time_since_sale_end = (current_time - (ctx.accounts.vesting.start_time + ctx.accounts.vesting.sale_duration as i64)) as u64;
        let vesting_amount = if time_since_sale_end >= ctx.accounts.vesting.vesting_duration {
            // If vesting period is complete, allow claiming full amount
            ctx.accounts.user_info.total_allocation
        } else {
            // Linear vesting for remaining 85% over 6 months
            let remaining_allocation = ctx.accounts.user_info.total_allocation.checked_sub(initial_allocation).unwrap();
            let vested_portion = remaining_allocation
                .checked_mul(time_since_sale_end)
                .unwrap()
                .checked_div(ctx.accounts.vesting.vesting_duration)
                .unwrap();
            initial_allocation.checked_add(vested_portion).unwrap()
        };

        // Check if requested amount is available
        let available_to_claim = vesting_amount.checked_sub(ctx.accounts.user_info.claimed_amount).unwrap();
        require!(
            amount <= available_to_claim,
            ErrorCode::AllocationAmountTooLarge
        );

        ctx.accounts.user_info.claimed_amount = ctx.accounts.user_info.claimed_amount.checked_add(amount).unwrap();

        // Transfer tokens from vault to user
        let seeds = &[ctx.accounts.vesting.to_account_info().key.as_ref(), &[ctx.accounts.vesting.bump]];
        let signer = &[&seeds[..]];
    
        token_2022::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.clone().to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_vault.clone().to_account_info(),
                    to: ctx.accounts.user_token.clone().to_account_info(),
                    mint: ctx.accounts.mint.clone().to_account_info(),
                    authority: ctx.accounts.vesting.clone().to_account_info(),
                },
                signer,
            ),
            amount,
            ctx.accounts.mint.decimals
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 8 * 6
    )]
    pub vesting: Account<'info, Vesting>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetVesting<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub vesting: Account<'info, Vesting>,                                   
    #[account(mut)]
    pub admin_token: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account()]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_program: Program<'info, Token2022>
}

#[derive(Accounts)]
pub struct AllowClaim<'info> {
    #[account(mut)]
    pub from: Signer<'info>,
    #[account(mut)]
    pub vesting: Account<'info, Vesting>,
    #[account(
        init_if_needed,
        payer = from,
        space = 8 + 8 * 3,
        seeds = [b"user_info", vesting.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    pub user: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub vesting: Account<'info, Vesting>,
    #[account(
        mut,
        seeds = [b"user_info", vesting.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_token: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token2022>,
}

#[account]
pub struct Vesting {
    pub start_time: i64,
    pub sale_duration: u64,
    pub vesting_duration: u64,
    pub amount: u64,
    pub investor_claimed: u64,
    pub community_claimed: u64,
    pub bump: u8,
}

#[account]
pub struct UserInfo {
    pub total_allocation: u64,
    pub claimed_amount: u64,
    pub is_investor: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Active vesting period exists")]
    ActiveVestingExists,
    #[msg("Allocation amount too large")]
    AllocationAmountTooLarge,
    #[msg("Sale not started")]
    SaleNotStarted,
    #[msg("Sale not ended")]
    SaleNotEnded,
}