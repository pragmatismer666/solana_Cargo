use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::collections::HashMap;

declare_id!("Fg6PaFhzVNhYgo5L5G5LsWKCukb8RNrFBLxe93B6tv1M");

#[program]
mod rev_gold {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, withdraw_fee: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.owner = *ctx.accounts.owner.key;
        state.withdraw_fee = withdraw_fee;
        Ok(())
    }

    pub fn update_tokens(ctx: Context<UpdateTokens>, tokens: Vec<Pubkey>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.available_tokens = tokens;
        Ok(())
    }

    pub fn add_admin(ctx: Context<AdminAction>, admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.admins.push(admin);
        Ok(())
    }

    pub fn remove_admin(ctx: Context<AdminAction>, admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.admins.retain(|&x| x != admin);
        Ok(())
    }

    pub fn update_fee(ctx: Context<AdminAction>, fee: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.withdraw_fee = fee;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let state = &mut ctx.accounts.state;
        let balance = state.balances.entry(ctx.accounts.token_mint.key()).or_insert(0);
        *balance += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let balance = state.balances.get_mut(&ctx.accounts.token_mint.key()).ok_or(ErrorCode::InsufficientFunds)?;
        require!(*balance >= amount, ErrorCode::InsufficientFunds);

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        *balance -= amount;
        Ok(())
    }
}

#[account]
pub struct State {
    pub owner: Pubkey,
    pub admins: Vec<Pubkey>,
    pub withdraw_fee: u64,
    pub available_tokens: Vec<Pubkey>,
    pub balances: HashMap<Pubkey, u64>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + 32 + (32 * 100) + 8 + (32 * 100) + 8)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTokens<'info> {
    #[account(mut, has_one = owner)]
    pub state: Account<'info, State>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(mut, has_one = owner)]
    pub state: Account<'info, State>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds")]
    InsufficientFunds,
}