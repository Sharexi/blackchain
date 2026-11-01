use std::collections::HashMap;
use crate::pricing::DynamicFeeMarket;
use crate::transaction::{StakeTx, TransferTx, UnstakeTx};
use crate::types::{Address, Amount, Nonce};
use blackchain_crypto::{calculate_merkle_root, hash, Hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Account {0} does not have sufficient balance")]
    InsufficientBalance(Address),
    #[error("Account {0} does not have sufficient staked balance")]
    InsufficientStake(Address),
    #[error("Invalid nonce for {account}: expected {expected}, got {got}")]
    InvalidNonce {
        account: Address,
        expected: Nonce,
        got: Nonce,
    },
    #[error("Account {0} has not met the minimum staking threshold to mine")]
    BelowMinimumStake(Address),
    #[error("Block height is invalid")]
    InvalidBlockHeight,
}

pub const MINIMUM_MINING_STAKE: Amount = 10_000_000_000_000_000_000; // 10 Tokens
pub const UNBONDING_PERIOD_BLOCKS: u64 = 100;
pub const TOTAL_GENESIS_SUPPLY: Amount = 1_000_000_000_000_000_000_000_000_000; // 1 Billion Tokens (10^18 precision)

/// Genesis state configuration defining initial token distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub allocations: HashMap<Address, Amount>,
}

impl GenesisConfig {
    /// Create a genesis configuration with a 10% founder allocation.
    pub fn with_founder_share(founder_address: Address) -> Self {
        let mut allocations = HashMap::new();
        // 10% of 1 Billion = 100 Million tokens
        let founder_share = TOTAL_GENESIS_SUPPLY / 10;
        allocations.insert(founder_address, founder_share);
        Self { allocations }
    }
}

/// Account balance, staking record, and nonce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Account {
    pub balance: Amount,
    pub staked: Amount,
    pub nonce: Nonce,
    pub unbonding_amount: Amount,
    pub unbonding_unlock_height: u64,
}

/// Ledger state tracking accounts and balances.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub accounts: HashMap<Address, Account>,
    pub fee_market: DynamicFeeMarket,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            fee_market: DynamicFeeMarket::default(),
        }
    }

    /// Initialize ledger state from genesis configuration with premined allocations.
    pub fn from_genesis(config: &GenesisConfig) -> Self {
        let mut state = Self::new();
        for (addr, amount) in &config.allocations {
            state.credit(addr, *amount);
        }
        state
    }

    pub fn get_account(&self, address: &Address) -> Account {
        self.accounts.get(address).cloned().unwrap_or_default()
    }

    pub fn credit(&mut self, address: &Address, amount: Amount) {
        let account = self.accounts.entry(*address).or_default();
        account.balance = account.balance.saturating_add(amount);
    }

    pub fn debit(&mut self, address: &Address, amount: Amount) -> Result<(), StateError> {
        let account = self.accounts.entry(*address).or_default();
        if account.balance < amount {
            return Err(StateError::InsufficientBalance(*address));
        }
        account.balance -= amount;
        Ok(())
    }

    pub fn apply_transfer(&mut self, tx: &TransferTx) -> Result<(), StateError> {
        let sender = self.get_account(&tx.from);
        if sender.nonce != tx.nonce {
            return Err(StateError::InvalidNonce {
                account: tx.from,
                expected: sender.nonce,
                got: tx.nonce,
            });
        }

        let total_deduction = tx.amount.checked_add(tx.fee).ok_or(StateError::InsufficientBalance(tx.from))?;
        if sender.balance < total_deduction {
            return Err(StateError::InsufficientBalance(tx.from));
        }

        // Debit sender
        let sender_mut = self.accounts.entry(tx.from).or_default();
        sender_mut.balance -= total_deduction;
        sender_mut.nonce += 1;

        // Credit recipient
        self.credit(&tx.to, tx.amount);

        Ok(())
    }

    pub fn apply_stake(&mut self, tx: &StakeTx) -> Result<(), StateError> {
        let sender = self.get_account(&tx.staker);
        if sender.nonce != tx.nonce {
            return Err(StateError::InvalidNonce {
                account: tx.staker,
                expected: sender.nonce,
                got: tx.nonce,
            });
        }

        let total_deduction = tx.amount.checked_add(tx.fee).ok_or(StateError::InsufficientBalance(tx.staker))?;
        if sender.balance < total_deduction {
            return Err(StateError::InsufficientBalance(tx.staker));
        }

        let account = self.accounts.entry(tx.staker).or_default();
        account.balance -= total_deduction;
        account.staked = account.staked.saturating_add(tx.amount);
        account.nonce += 1;

        Ok(())
    }

    pub fn apply_unstake(&mut self, tx: &UnstakeTx, current_height: u64) -> Result<(), StateError> {
        let sender = self.get_account(&tx.staker);
        if sender.nonce != tx.nonce {
            return Err(StateError::InvalidNonce {
                account: tx.staker,
                expected: sender.nonce,
                got: tx.nonce,
            });
        }

        if sender.staked < tx.amount {
            return Err(StateError::InsufficientStake(tx.staker));
        }

        let account = self.accounts.entry(tx.staker).or_default();
        account.staked -= tx.amount;
        account.unbonding_amount = account.unbonding_amount.saturating_add(tx.amount);
        account.unbonding_unlock_height = current_height + UNBONDING_PERIOD_BLOCKS;
        account.nonce += 1;

        Ok(())
    }

    /// Calculate the state root hash across all active accounts
    pub fn calculate_state_root(&self) -> Hash {
        let mut account_hashes: Vec<Hash> = Vec::with_capacity(self.accounts.len());

        // Sort keys to maintain deterministic root calculation
        let mut sorted_addrs: Vec<&Address> = self.accounts.keys().collect();
        sorted_addrs.sort();

        for addr in sorted_addrs {
            let acc = &self.accounts[addr];
            let mut bytes = Vec::new();
            bytes.extend_from_slice(addr.as_bytes());
            bytes.extend_from_slice(&acc.balance.to_be_bytes());
            bytes.extend_from_slice(&acc.staked.to_be_bytes());
            bytes.extend_from_slice(&acc.nonce.to_be_bytes());
            account_hashes.push(hash(&bytes));
        }

        calculate_merkle_root(&account_hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_debit() {
        let mut state = State::new();
        let addr = Address([1u8; 32]);

        state.credit(&addr, 1000);
        assert_eq!(state.get_account(&addr).balance, 1000);

        assert!(state.debit(&addr, 400).is_ok());
        assert_eq!(state.get_account(&addr).balance, 600);

        assert!(state.debit(&addr, 700).is_err());
    }

    #[test]
    fn test_deterministic_state_root() {
        let mut s1 = State::new();
        let mut s2 = State::new();

        let a1 = Address([1u8; 32]);
        let a2 = Address([2u8; 32]);

        s1.credit(&a1, 500);
        s1.credit(&a2, 1000);

        // Insert in reverse order into s2
        s2.credit(&a2, 1000);
        s2.credit(&a1, 500);

        assert_eq!(s1.calculate_state_root(), s2.calculate_state_root());
    }
}
