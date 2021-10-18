use crate::amount_type::AmountType;
use std::error::Error;
use std::fmt::Formatter;

type Result<T> = std::result::Result<(), T>;

#[derive(Debug, Clone, PartialEq)]
pub struct DepositError;

impl std::fmt::Display for DepositError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Couldn't deposit due to maximum funds has been reached!")
    }
}

impl Error for DepositError {}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputeError;

impl std::fmt::Display for DisputeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Couldn't dispute funds due to reaching maximum held funds or maximum debit!"
        )
    }
}

impl Error for DisputeError {}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolveError;

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Couldn't withdraw due to locked account or there is no sufficient funds!"
        )
    }
}

impl Error for ResolveError {}

#[derive(Debug, PartialEq)]
pub struct ClientAccount {
    pub available: AmountType,
    pub held: AmountType,
    pub locked: bool,
}

impl Default for ClientAccount {
    fn default() -> Self {
        Self {
            available: 0,
            held: 0,
            locked: false,
        }
    }
}

impl ClientAccount {
    /// Returns the total funds
    pub fn total(&self) -> AmountType {
        self.available + self.held
    }

    /// Increases available funds.
    /// If the account is locked the operation is dropped.
    /// Returns a DepositError when funds cannot be increased due to overflow.
    /// # Arguments
    /// * `amount` - the amount that will be added to the available funds
    pub fn deposit(&mut self, amount: AmountType) -> Result<DepositError> {
        if !self.locked {
            if let Some(new_available) = self.available.checked_add(amount) {
                self.available = new_available;
            } else {
                return Err(DepositError);
            }
        }
        Ok(())
    }

    /// Decreases the available funds.
    /// If the account is locked or there is no sufficient funds drop the operation.
    /// # Arguments
    /// * `amount` - the amount that will be subtracted from the available funds
    pub fn withdraw(&mut self, amount: AmountType) {
        if !self.locked && self.available >= amount {
            self.available -= amount;
        }
    }

    /// Moves the funds from the available to the held ones.
    /// Returns a DisputeError when the available funds can't be debit
    /// anymore or the held funds are going to be overflown!
    /// If account is locked the operation doesn't take effect.
    /// # Arguments
    /// * `amount` - the amount that will be moved
    pub fn dispute(&mut self, amount: AmountType) -> Result<DisputeError> {
        if !self.locked {
            let sub_result = self.available.checked_sub(amount);
            let add_result = self.held.checked_add(amount);
            if let (Some(new_available), Some(new_held)) = (sub_result, add_result) {
                self.available = new_available;
                self.held = new_held;
            } else {
                return Err(DisputeError);
            }
        }
        Ok(())
    }

    /// Moves the funds from the held to the available ones.
    /// If account is locked the operation doesn't take effect.
    /// Returns a ResolveError when the available funds are going to be overflown!
    /// # Arguments
    /// * `amount` - the amount that will be moved
    pub fn resolve(&mut self, amount: AmountType) -> Result<ResolveError> {
        if !self.locked {
            let sub_result = self.held.checked_sub(amount);
            let add_result = self.available.checked_add(amount);
            if let (Some(new_held), Some(new_available)) = (sub_result, add_result) {
                self.available = new_available;
                self.held = new_held;
            } else {
                return Err(ResolveError);
            }
        }
        Ok(())
    }

    /// Decreases the held funds and locks the account.
    /// If account is already locked the operation doesn't take effect.
    /// # Arguments
    /// * `amount` - the amount that will be subtracted from the held funds
    pub fn chargeback(&mut self, amount: AmountType) {
        if !self.locked {
            self.held -= amount;
            self.locked = true;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deposit_to_client_account() {
        let mut account = ClientAccount::default();

        assert_eq!(account.deposit(10), Ok(()));
        assert_eq!(account.total(), 10);
        assert_eq!(
            account,
            ClientAccount {
                available: 10,
                held: 0,
                locked: false
            }
        );

        assert_eq!(account.deposit(100), Ok(()));
        assert_eq!(account.total(), 110);
        assert_eq!(
            account,
            ClientAccount {
                available: 110,
                held: 0,
                locked: false
            }
        );

        assert_eq!(account.deposit(AmountType::MAX), Err(DepositError));
        assert_eq!(account.total(), 110);
        assert_eq!(
            account,
            ClientAccount {
                available: 110,
                held: 0,
                locked: false
            }
        );

        account.locked = true;
        assert_eq!(account.deposit(100), Ok(()));
        assert_eq!(account.total(), 110);
        assert_eq!(
            account,
            ClientAccount {
                available: 110,
                held: 0,
                locked: true
            }
        );
    }

    #[test]
    fn test_withdraw_from_client_account() {
        let mut account = ClientAccount {
            available: 1000,
            held: 1000,
            locked: false,
        };

        account.withdraw(100);
        assert_eq!(account.total(), 1900);
        assert_eq!(
            account,
            ClientAccount {
                available: 900,
                held: 1000,
                locked: false
            }
        );

        account.withdraw(800);
        assert_eq!(account.total(), 1100);
        assert_eq!(
            account,
            ClientAccount {
                available: 100,
                held: 1000,
                locked: false
            }
        );

        account.withdraw(200);
        assert_eq!(account.total(), 1100);
        assert_eq!(
            account,
            ClientAccount {
                available: 100,
                held: 1000,
                locked: false
            }
        );

        account.locked = true;
        account.withdraw(100);
        assert_eq!(account.total(), 1100);
        assert_eq!(
            account,
            ClientAccount {
                available: 100,
                held: 1000,
                locked: true
            }
        );
    }

    #[test]
    fn test_dispute_client_account() {
        let mut account = ClientAccount {
            available: 1000,
            held: 0,
            locked: false,
        };

        assert_eq!(account.dispute(100), Ok(()));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: 900,
                held: 100,
                locked: false
            }
        );

        assert_eq!(account.dispute(1000), Ok(()));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: -100,
                held: 1100,
                locked: false
            }
        );

        //Overflow the held funds
        assert_eq!(account.dispute(AmountType::MAX - 1000), Err(DisputeError));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: -100,
                held: 1100,
                locked: false
            }
        );

        account.locked = true;
        assert_eq!(account.dispute(50), Ok(()));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: -100,
                held: 1100,
                locked: true
            }
        );
    }

    #[test]
    fn test_resolve_client_account() {
        let mut account = ClientAccount {
            available: 0,
            held: 1000,
            locked: false,
        };

        assert_eq!(account.resolve(100), Ok(()));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: 100,
                held: 900,
                locked: false
            }
        );

        assert_eq!(account.resolve(1000), Ok(()));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: 1100,
                held: -100,
                locked: false
            }
        );

        //Overflow the available and the held funds
        assert_eq!(account.resolve(AmountType::MAX), Err(ResolveError));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: 1100,
                held: -100,
                locked: false
            }
        );

        account.locked = true;
        assert_eq!(account.resolve(50), Ok(()));
        assert_eq!(account.total(), 1000);
        assert_eq!(
            account,
            ClientAccount {
                available: 1100,
                held: -100,
                locked: true
            }
        );
    }

    #[test]
    fn test_chargeback_client_account() {
        let mut account = ClientAccount {
            available: 0,
            held: 1000,
            locked: false,
        };

        account.chargeback(100);
        assert_eq!(account.total(), 900);
        assert_eq!(
            account,
            ClientAccount {
                available: 0,
                held: 900,
                locked: true
            }
        );

        account.chargeback(1000);
        assert_eq!(account.total(), 900);
        assert_eq!(
            account,
            ClientAccount {
                available: 0,
                held: 900,
                locked: true
            }
        );
    }
}
