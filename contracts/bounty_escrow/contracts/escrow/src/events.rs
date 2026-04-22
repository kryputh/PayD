/// Event emitted when admin rotation is proposed
#[ink::event]
pub struct AdminRotationProposed {
    #[ink(topic)]
    old_admin: AccountId,
    #[ink(topic)]
    new_admin: AccountId,
    unlock_time: Timestamp,
}

/// Event emitted when admin rotation is accepted
#[ink::event]
pub struct AdminRotationAccepted {
    #[ink(topic)]
    old_admin: AccountId,
    #[ink(topic)]
    new_admin: AccountId,
    timestamp: Timestamp,
}

/// Event emitted when admin rotation is cancelled
#[ink::event]
pub struct AdminRotationCancelled {
    #[ink(topic)]
    admin: AccountId,
    #[ink(topic)]
    pending_admin: AccountId,
}