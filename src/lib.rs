use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::collections::HashMap;

declare_id!("Fg6PaFhzVNhYgo5L5G5LsWKCukb8RNrFBLxe93B6tv1M");

#[program]
pub mod rev_gold {
    use super::*;
    
    pub fn initialize(ctx: Context<Initialize>, withdraw_fee: u64) -> Result<()> {
        let rev_gold = &mut ctx.accounts.rev_gold;
        rev_gold.withdraw_fee = withdraw_fee;
        rev_gold.owner = *ctx.accounts.admin.key;  // Setting owner as the admin's key
        rev_gold.admins.push(*ctx.accounts.admin.key);
        Ok(())
    }

    pub fn update_tokens(ctx: Context<UpdateTokens>, tokens: Vec<Pubkey>) -> Result<()> {
        let rev_gold = &mut ctx.accounts.rev_gold;
        require!(rev_gold.admins.contains(&ctx.accounts.admin.key()), ErrorCode::Unauthorized);
        rev_gold.available_tokens = tokens;
        Ok(())
    }

    pub fn add_admin(ctx: Context<AdminOps>, new_admin: Pubkey) -> Result<()> {
        let rev_gold = &mut ctx.accounts.rev_gold;
        require!(rev_gold.owner == *ctx.accounts.admin.key, ErrorCode::Unauthorized);
        rev_gold.admins.push(new_admin);
        Ok(())
    }

    pub fn remove_admin(ctx: Context<AdminOps>, admin: Pubkey) -> Result<()> {
        let rev_gold = &mut ctx.accounts.rev_gold;
        require!(rev_gold.owner == *ctx.accounts.admin.key, ErrorCode::Unauthorized);
        rev_gold.admins.retain(|&x| x != admin);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let rev_gold = &mut ctx.accounts.rev_gold;
        let _user_key = *ctx.accounts.user.key;
        let token_key = *ctx.accounts.token_account.key;
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        *rev_gold.balances.entry(token_key).or_insert(0) += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let rev_gold = &mut ctx.accounts.rev_gold;
        let token_key = *ctx.accounts.token_account.key;
        let balance = rev_gold.balances.get_mut(&token_key).ok_or(ErrorCode::InsufficientFunds)?;
        require!(*balance >= amount, ErrorCode::InsufficientFunds);

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        *balance -= amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 40 + 4 + (32 * 100) + 1024)] // Adjusted space to accommodate the HashMap
    pub rev_gold: Account<'info, RevGold>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTokens<'info> {
    // #[account(mut, has_one = owner)]  // Ensure the owner field is accessed correctly
    pub rev_gold: Account<'info, RevGold>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdminOps<'info> {
    #[account(mut)]
    pub rev_gold: Account<'info, RevGold>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub rev_gold: Account<'info, RevGold>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub rev_gold: Account<'info, RevGold>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct RevGold {
    pub owner: Pubkey,
    pub admins: Vec<Pubkey>,
    pub available_tokens: Vec<Pubkey>,
    pub balances: HashMap<Pubkey, u64>,
    pub withdraw_fee: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Insufficient funds for withdrawal.")]
    InsufficientFunds,
}