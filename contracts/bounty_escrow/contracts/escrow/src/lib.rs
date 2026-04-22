#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod bounty_escrow {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    /// Defines the storage of your contract.
    #[ink(storage)]
    pub struct BountyEscrow {
        /// Current admin
        admin: AccountId,
        /// Pending admin for rotation
        pending_admin: Option<AccountId>,
        /// Timelock timestamp for admin rotation
        admin_timelock: Option<Timestamp>,
        /// Timelock duration in seconds
        timelock_duration: Timestamp,
    }

    /// Event emitted when admin rotation is proposed
    #[ink(event)]
    pub struct AdminRotationProposed {
        #[ink(topic)]
        old_admin: AccountId,
        #[ink(topic)]
        new_admin: AccountId,
        unlock_time: Timestamp,
    }

    /// Event emitted when admin rotation is accepted
    #[ink(event)]
    pub struct AdminRotationAccepted {
        #[ink(topic)]
        old_admin: AccountId,
        #[ink(topic)]
        new_admin: AccountId,
        timestamp: Timestamp,
    }

    /// Event emitted when admin rotation is cancelled
    #[ink(event)]
    pub struct AdminRotationCancelled {
        #[ink(topic)]
        admin: AccountId,
        #[ink(topic)]
        pending_admin: AccountId,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Unauthorized access
        Unauthorized,
        /// No pending admin rotation
        NoPendingRotation,
        /// Timelock has not expired yet
        TimelockNotExpired,
        /// Cannot propose self as new admin
        InvalidAdmin,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl BountyEscrow {
        /// Constructor that initializes the contract.
        #[ink(constructor)]
        pub fn new(admin: AccountId, timelock_duration: Timestamp) -> Self {
            Self {
                admin,
                pending_admin: None,
                admin_timelock: None,
                timelock_duration,
            }
        }

        /// Propose a new admin with timelock. Only current admin can propose.
        #[ink(message)]
        pub fn propose_admin_rotation(&mut self, new_admin: AccountId) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::Unauthorized);
            }

            if new_admin == self.admin {
                return Err(Error::InvalidAdmin);
            }

            let unlock_time = self.env().block_timestamp() + self.timelock_duration;

            self.pending_admin = Some(new_admin);
            self.admin_timelock = Some(unlock_time);

            self.env().emit_event(AdminRotationProposed {
                old_admin: self.admin,
                new_admin,
                unlock_time,
            });

            Ok(())
        }

        /// Accept the proposed admin rotation after timelock expires.
        #[ink(message)]
        pub fn accept_admin_rotation(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let pending_admin = self.pending_admin.ok_or(Error::NoPendingRotation)?;

            if caller != pending_admin {
                return Err(Error::Unauthorized);
            }

            let unlock_time = self.admin_timelock.ok_or(Error::NoPendingRotation)?;

            if self.env().block_timestamp() < unlock_time {
                return Err(Error::TimelockNotExpired);
            }

            let old_admin = self.admin;
            self.admin = pending_admin;
            self.pending_admin = None;
            self.admin_timelock = None;

            self.env().emit_event(AdminRotationAccepted {
                old_admin,
                new_admin: self.admin,
                timestamp: self.env().block_timestamp(),
            });

            Ok(())
        }

        /// Cancel pending admin rotation. Only current admin can cancel.
        #[ink(message)]
        pub fn cancel_admin_rotation(&mut self) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::Unauthorized);
            }

            let pending_admin = self.pending_admin.ok_or(Error::NoPendingRotation)?;

            self.pending_admin = None;
            self.admin_timelock = None;

            self.env().emit_event(AdminRotationCancelled {
                admin: self.admin,
                pending_admin,
            });

            Ok(())
        }

        /// Get current admin
        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            self.admin
        }

        /// Get pending admin and timelock
        #[ink(message)]
        pub fn get_pending_admin(&self) -> Option<(AccountId, Timestamp)> {
            match (self.pending_admin, self.admin_timelock) {
                (Some(admin), Some(timelock)) => Some((admin, timelock)),
                _ => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_admin_rotation() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = BountyEscrow::new(accounts.alice, 86400); // 24 hours

            // Propose rotation
            assert_eq!(contract.propose_admin_rotation(accounts.bob), Ok(()));

            // Check pending admin
            assert_eq!(contract.get_pending_admin(), Some((accounts.bob, contract.env().block_timestamp() + 86400)));

            // Try to accept too early
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(contract.accept_admin_rotation(), Err(Error::TimelockNotExpired));

            // Advance time
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();

            // Accept rotation
            assert_eq!(contract.accept_admin_rotation(), Ok(()));
            assert_eq!(contract.get_admin(), accounts.bob);
        }
    }
}