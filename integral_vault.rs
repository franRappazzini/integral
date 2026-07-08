// integral_vault.rs
// Integral: bribe/incentive vault para outcome tokens de prediction markets (estilo Reflex).
// Anti-whale por diseño: reward = integral(peso * tiempo), NO snapshot ni pro-rata al claim.
// Defensas: (1) accrual continuo tipo MasterChef, (2) claim gated a resolucion,
//           (3) forfeit del reward no vesteado si salis antes de resolver.
//
// Stack: Anchor + anchor_spl::token_interface  (sirve para SPL clasico y Token-2022 / CASH).

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

declare_id!("Integ1111111111111111111111111111111111111");

/// Escala fija para acc_reward_per_share. 1e12. Nada de floats en BPF.
const SCALE: u128 = 1_000_000_000_000;

#[program]
pub mod integral_vault {
    use super::*;

    // ─────────────────────────────────────────────────────────────
    // init_vault: crea un vault por outcome (uno para Argentina, otro para Francia).
    // reward_rate debe fondearse como  bribe_reserve / (epoch_end - now).
    // ─────────────────────────────────────────────────────────────
    pub fn init_vault(ctx: Context<InitVault>, reward_rate: u64, epoch_end: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(epoch_end > now, IntegralError::BadEpoch);

        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.outcome_mint = ctx.accounts.outcome_mint.key();
        vault.reward_mint = ctx.accounts.reward_mint.key();
        vault.vault_token = ctx.accounts.vault_token.key();
        vault.reward_vault = ctx.accounts.reward_vault.key();
        vault.total_shares = 0;
        vault.acc_reward_per_share = 0;
        vault.last_update_ts = now;
        vault.reward_rate = reward_rate;
        vault.epoch_end = epoch_end;
        vault.resolved = false;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // deposit: mete outcome tokens al vault.
    // El token entra (inbound), asi que el riesgo de reentrancy por transfer-hook
    // es menor, pero igual banqueamos el pending y actualizamos debt de forma limpia.
    // No se puede depositar despues de resolved.
    // ─────────────────────────────────────────────────────────────
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, IntegralError::ZeroAmount);
        let now = Clock::get()?.unix_timestamp;

        let vault = &mut ctx.accounts.vault;
        require!(!vault.resolved, IntegralError::AlreadyResolved);
        vault.accrue(now)?;

        let position = &mut ctx.accounts.position;

        if position.shares > 0 {
            // banca el pending acumulado hasta ahora (se paga recien al claim, post-resolucion)
            let pending = vault.pending(position);
            position.accrued = position
                .accrued
                .checked_add(pending as u64)
                .ok_or(IntegralError::MathOverflow)?;
        } else {
            // primera vez: inicializa la posicion
            position.owner = ctx.accounts.owner.key();
            position.vault = vault.key();
            position.deposit_ts = now; // para warmup/analytics; el forfeit no depende de esto
        }

        // interactions: token entra del usuario al vault_token
        let decimals = ctx.accounts.outcome_mint.decimals;
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.owner_token.to_account_info(),
                    mint: ctx.accounts.outcome_mint.to_account_info(),
                    to: ctx.accounts.vault_token.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
            decimals,
        )?;

        // effects
        position.shares = position
            .shares
            .checked_add(amount)
            .ok_or(IntegralError::MathOverflow)?;
        vault.total_shares = vault
            .total_shares
            .checked_add(amount)
            .ok_or(IntegralError::MathOverflow)?;
        position.settle_debt(vault)?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // withdraw: saca outcome tokens.
    //
    // CLAVE anti-whale: si !resolved, TODO el reward no vesteado (accrued + pending)
    // se FORFEITEA y se redistribuye subiendo acc_reward_per_share sobre los que quedan.
    // El que sale se lleva SOLO su principal. El whale JIT no gana: subsidia a los honestos.
    //
    // Post-resolution el withdraw es normal: banca el pending para claimear.
    //
    // Politica: forfeit total sobre CUALQUIER salida anticipada (simple y duro).
    // Si preferis forfeit pro-rata al monto retirado, ver nota abajo.
    // ─────────────────────────────────────────────────────────────
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, IntegralError::ZeroAmount);
        let now = Clock::get()?.unix_timestamp;

        let vault = &mut ctx.accounts.vault;
        vault.accrue(now)?;

        let position = &mut ctx.accounts.position;
        require!(position.shares >= amount, IntegralError::InsufficientShares);

        let pending = vault.pending(position);

        if !vault.resolved {
            // FORFEIT: reward no vesteado del que sale -> pool de los que quedan
            let forfeit = pending
                .checked_add(position.accrued as u128)
                .ok_or(IntegralError::MathOverflow)?;
            position.accrued = 0;

            let remaining = (vault.total_shares as u128)
                .checked_sub(amount as u128)
                .ok_or(IntegralError::MathOverflow)?;

            if remaining > 0 && forfeit > 0 {
                let bump = forfeit
                    .checked_mul(SCALE)
                    .ok_or(IntegralError::MathOverflow)?
                    .checked_div(remaining)
                    .ok_or(IntegralError::MathOverflow)?;
                vault.acc_reward_per_share = vault
                    .acc_reward_per_share
                    .checked_add(bump)
                    .ok_or(IntegralError::MathOverflow)?;
            }
            // si remaining == 0 (el ultimo sale), el forfeit se pierde / mandalo a treasury.
        } else {
            // resuelto: banca normal, claimeable
            position.accrued = position
                .accrued
                .checked_add(pending as u64)
                .ok_or(IntegralError::MathOverflow)?;
        }

        // effects PRIMERO (checks-effects-interactions: mandamos tokens AL usuario despues)
        position.shares = position
            .shares
            .checked_sub(amount)
            .ok_or(IntegralError::MathOverflow)?;
        vault.total_shares = vault
            .total_shares
            .checked_sub(amount)
            .ok_or(IntegralError::MathOverflow)?;
        position.settle_debt(vault)?;

        // interactions: vault PDA firma la salida de outcome tokens
        let outcome_mint_key = vault.outcome_mint;
        let bump = vault.bump;
        let seeds: &[&[u8]] = &[b"vault", outcome_mint_key.as_ref(), &[bump]];
        let signer: &[&[&[u8]]] = &[seeds];

        let decimals = ctx.accounts.outcome_mint.decimals;
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault_token.to_account_info(),
                    mint: ctx.accounts.outcome_mint.to_account_info(),
                    to: ctx.accounts.owner_token.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                signer,
            ),
            amount,
            decimals,
        )?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // resolve: solo authority (o el oracle adapter, ej. World / Chainlink).
    // Congela accrual y abre el claim. Idempotente-ish: revert si ya esta resuelto.
    // ─────────────────────────────────────────────────────────────
    pub fn resolve(ctx: Context<Resolve>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let vault = &mut ctx.accounts.vault;
        require_keys_eq!(
            ctx.accounts.authority.key(),
            vault.authority,
            IntegralError::Unauthorized
        );
        require!(!vault.resolved, IntegralError::AlreadyResolved);
        vault.accrue(now)?;
        vault.resolved = true;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // claim: SOLO post-resolucion. Paga reward tokens (CASH) desde reward_vault.
    // Antes de resolved no hay lump para agarrar -> "entro y salgo en 10 min" no extrae nada.
    // ─────────────────────────────────────────────────────────────
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let vault = &mut ctx.accounts.vault;
        require!(vault.resolved, IntegralError::NotResolved);
        vault.accrue(now)?;

        let position = &mut ctx.accounts.position;
        let pending = vault.pending(position);
        let payout = position
            .accrued
            .checked_add(pending as u64)
            .ok_or(IntegralError::MathOverflow)?;
        require!(payout > 0, IntegralError::NothingToClaim);

        // effects primero
        position.accrued = 0;
        position.settle_debt(vault)?;

        // interactions: reward_vault (PDA vault firma) -> usuario
        let outcome_mint_key = vault.outcome_mint;
        let bump = vault.bump;
        let seeds: &[&[u8]] = &[b"vault", outcome_mint_key.as_ref(), &[bump]];
        let signer: &[&[&[u8]]] = &[seeds];

        let decimals = ctx.accounts.reward_mint.decimals;
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.reward_vault.to_account_info(),
                    mint: ctx.accounts.reward_mint.to_account_info(),
                    to: ctx.accounts.owner_reward.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                signer,
            ),
            payout,
            decimals,
        )?;
        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════
// STATE
// ══════════════════════════════════════════════════════════════════

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub authority: Pubkey,
    pub outcome_mint: Pubkey, // token que se stakea (outcome token)
    pub reward_mint: Pubkey,  // token de reward / bribe (CASH)
    pub vault_token: Pubkey,  // ATA del vault para outcome tokens
    pub reward_vault: Pubkey, // ATA del vault para reward tokens
    pub total_shares: u64,
    pub acc_reward_per_share: u128, // escalado por SCALE (1e12)
    pub last_update_ts: i64,        // Clock.unix_timestamp
    pub reward_rate: u64,           // reward por segundo
    pub epoch_end: i64,             // corta el accrual (accrue capea now a esto)
    pub resolved: bool,
    pub bump: u8,
}

impl Vault {
    /// Actualiza el acumulador global. Esto ES la integral peso*tiempo. O(1), sin loops.
    /// Capea `now` a epoch_end para que el reward no siga corriendo pasado el epoch.
    fn accrue(&mut self, now: i64) -> Result<()> {
        let capped = now.min(self.epoch_end);
        if self.total_shares == 0 || capped <= self.last_update_ts {
            self.last_update_ts = self.last_update_ts.max(capped);
            return Ok(());
        }

        // dt puede "retroceder" unos segundos entre validadores -> el guard de arriba lo cubre
        let dt = (capped - self.last_update_ts) as u128;
        let reward = dt
            .checked_mul(self.reward_rate as u128)
            .ok_or(IntegralError::MathOverflow)?;
        // ESCALAR ANTES DE DIVIDIR, si no truncas y perdes reward
        let delta = reward
            .checked_mul(SCALE)
            .ok_or(IntegralError::MathOverflow)?
            .checked_div(self.total_shares as u128)
            .ok_or(IntegralError::MathOverflow)?;
        self.acc_reward_per_share = self
            .acc_reward_per_share
            .checked_add(delta)
            .ok_or(IntegralError::MathOverflow)?;
        self.last_update_ts = capped;
        Ok(())
    }

    /// Reward pendiente de una posicion en el instante actual (post-accrue).
    fn pending(&self, p: &Position) -> u128 {
        (p.shares as u128)
            .saturating_mul(self.acc_reward_per_share)
            .checked_div(SCALE)
            .unwrap_or(0)
            .saturating_sub(p.reward_debt)
    }
}

#[account]
#[derive(InitSpace)]
pub struct Position {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub shares: u64,
    pub reward_debt: u128, // shares * acc / SCALE en el ultimo toque
    pub accrued: u64,      // reward banqueado, claimeable recien post-resolucion
    pub deposit_ts: i64,
    pub bump: u8,
}

impl Position {
    /// Resetea el debt al acc actual. Llamar SIEMPRE despues de cambiar shares.
    fn settle_debt(&mut self, vault: &Vault) -> Result<()> {
        self.reward_debt = (self.shares as u128)
            .checked_mul(vault.acc_reward_per_share)
            .ok_or(IntegralError::MathOverflow)?
            .checked_div(SCALE)
            .ok_or(IntegralError::MathOverflow)?;
        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════
// CONTEXTS
// ══════════════════════════════════════════════════════════════════

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", outcome_mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    pub outcome_mint: InterfaceAccount<'info, Mint>,
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(token::mint = outcome_mint, token::authority = vault)]
    pub vault_token: InterfaceAccount<'info, TokenAccount>,
    #[account(token::mint = reward_mint, token::authority = vault)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.outcome_mint.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + Position::INIT_SPACE,
        seeds = [b"position", vault.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, token::mint = vault.outcome_mint, token::authority = owner)]
    pub owner_token: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = vault.vault_token)]
    pub vault_token: InterfaceAccount<'info, TokenAccount>,

    #[account(address = vault.outcome_mint)]
    pub outcome_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.outcome_mint.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"position", vault.key().as_ref(), owner.key().as_ref()],
        bump = position.bump,
        has_one = owner
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, token::mint = vault.outcome_mint, token::authority = owner)]
    pub owner_token: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = vault.vault_token)]
    pub vault_token: InterfaceAccount<'info, TokenAccount>,

    #[account(address = vault.outcome_mint)]
    pub outcome_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Resolve<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.outcome_mint.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.outcome_mint.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"position", vault.key().as_ref(), owner.key().as_ref()],
        bump = position.bump,
        has_one = owner
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, address = vault.reward_vault)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, token::mint = vault.reward_mint, token::authority = owner)]
    pub owner_reward: InterfaceAccount<'info, TokenAccount>,

    #[account(address = vault.reward_mint)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

// ══════════════════════════════════════════════════════════════════
// ERRORS
// ══════════════════════════════════════════════════════════════════

#[error_code]
pub enum IntegralError {
    #[msg("epoch_end debe ser futuro")]
    BadEpoch,
    #[msg("amount debe ser > 0")]
    ZeroAmount,
    #[msg("vault ya resuelto")]
    AlreadyResolved,
    #[msg("vault no resuelto todavia")]
    NotResolved,
    #[msg("shares insuficientes")]
    InsufficientShares,
    #[msg("nada para claimear")]
    NothingToClaim,
    #[msg("no autorizado")]
    Unauthorized,
    #[msg("overflow aritmetico")]
    MathOverflow,
}

// ══════════════════════════════════════════════════════════════════
// NOTAS
// ══════════════════════════════════════════════════════════════════
//
// 1. FONDEO DEL REWARD: reward_rate = bribe_reserve / (epoch_end - start_ts).
//    accrue() capea `now` a epoch_end, asi que nunca distribuis mas que el reserve.
//    Verificar off-chain que reward_vault tenga >= reserve antes de abrir depositos.
//
// 2. FORFEIT PRO-RATA (alternativa al forfeit total):
//    Si un partial withdraw no deberia matar TODO el accrued, forfeitea solo la fraccion
//    retirada:  forfeit = pending * amount / shares_pre  +  accrued * amount / shares_pre.
//    Mas justo para el honesto que hace un retiro chico; menos disuasivo para el whale.
//    El modelo de este archivo es forfeit TOTAL (mas simple, mas duro). Es un knob de politica.
//
// 3. WARMUP (opcional, encima del forfeit): peso efectivo = shares * min(1, (now-deposit_ts)/RAMP).
//    Penaliza no-linealmente las tenencias cortas. Ojo con el costo en CU si lo agregas al accrue.
//    NUNCA iteres sobre depositantes: el diseño entero se cae si haces loops por holder.
//
// 4. PRECISION: si total_shares es chico y reward grande, el escalado 1e12 puede aun perder
//    "polvo" por truncamiento. Es dust y no exploitable; se acumula en el vault. Aceptable.
//
// 5. TOKEN-2022: transfer_checked ya soporta hooks/fees. Si el outcome token tiene transfer-fee,
//    el `amount` recibido en vault_token puede ser menor: si vas a soportar fee-on-transfer,
//    media el balance antes/despues en vez de confiar en `amount`. Para CASH normal no aplica.
//
// 6. CIERRE DE POSICION: agregar un ix close_position (post-resolucion, shares==0, accrued==0)
//    con `close = owner` para recuperar el rent de la Position PDA.
//
// 7. TWO-VAULT (Argentina / Francia): son dos Vault PDAs independientes, cada uno con su
//    outcome_mint como seed. El bribe se rutea por outcome. Un depositante puede tener
//    posiciones en ambos; son PDAs distintas, no se pisan.
