#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contractevent, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct AdminConfig {
    pub current_admin: Address,
    pub pending_admin: Option<Address>,
    pub admin_timelock: Option<u64>,
    pub timelock_duration: u64,
}

#[contracttype]
pub enum DataKey {
    AdminConfig,
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Emitted when admin rotation is proposed
#[contractevent]
pub struct AdminRotationProposed(
    pub Address, // current_admin
    pub Address, // proposed_admin
    pub u64,     // timelock_expiry
);

/// Emitted when admin rotation is accepted
#[contractevent]
pub struct AdminRotationAccepted(
    pub Address, // old_admin
    pub Address, // new_admin
);

/// Emitted when admin rotation is cancelled
#[contractevent]
pub struct AdminRotationCancelled(
    pub Address, // current_admin
    pub Address, // cancelled_admin
);

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct BountyEscrow;

#[contractimpl]
impl BountyEscrow {
    /// Initialize the contract with an admin and timelock duration
    pub fn initialize(env: Env, admin: Address, timelock_duration: u64) {
        if env.storage().instance().has(&DataKey::AdminConfig) {
            panic!("Contract already initialized");
        }

        let config = AdminConfig {
            current_admin: admin,
            pending_admin: None,
            admin_timelock: None,
            timelock_duration,
        };

        env.storage().instance().set(&DataKey::AdminConfig, &config);
    }

    /// Propose a new admin with timelock
    pub fn propose_admin_rotation(env: Env, caller: Address, new_admin: Address) {
        caller.require_auth();

        let mut config: AdminConfig = env.storage().instance().get(&DataKey::AdminConfig)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if caller != config.current_admin {
            panic!("Only current admin can propose rotation");
        }

        if config.pending_admin.is_some() {
            panic!("Admin rotation already pending");
        }

        let timelock_expiry = env.ledger().timestamp() + config.timelock_duration;

        config.pending_admin = Some(new_admin.clone());
        config.admin_timelock = Some(timelock_expiry);

        env.storage().instance().set(&DataKey::AdminConfig, &config);

        env.events().publish(
            (Symbol::new(&env, "admin_rotation_proposed"),),
            AdminRotationProposed(config.current_admin, new_admin, timelock_expiry),
        );
    }

    /// Accept the pending admin rotation after timelock
    pub fn accept_admin_rotation(env: Env, caller: Address) {
        caller.require_auth();

        let mut config: AdminConfig = env.storage().instance().get(&DataKey::AdminConfig)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        let pending_admin = config.pending_admin
            .clone()
            .unwrap_or_else(|| panic!("No pending admin rotation"));

        if caller != pending_admin {
            panic!("Only pending admin can accept rotation");
        }

        let timelock_expiry = config.admin_timelock
            .unwrap_or_else(|| panic!("No timelock set"));

        if env.ledger().timestamp() < timelock_expiry {
            panic!("Timelock not expired");
        }

        let old_admin = config.current_admin.clone();
        config.current_admin = pending_admin.clone();
        config.pending_admin = None;
        config.admin_timelock = None;

        env.storage().instance().set(&DataKey::AdminConfig, &config);

        env.events().publish(
            (Symbol::new(&env, "admin_rotation_accepted"),),
            AdminRotationAccepted(old_admin, pending_admin),
        );
    }

    /// Cancel the pending admin rotation
    pub fn cancel_admin_rotation(env: Env, caller: Address) {
        caller.require_auth();

        let mut config: AdminConfig = env.storage().instance().get(&DataKey::AdminConfig)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if caller != config.current_admin {
            panic!("Only current admin can cancel rotation");
        }

        if config.pending_admin.is_none() {
            panic!("No pending admin rotation to cancel");
        }

        let cancelled_admin = config.pending_admin.clone().unwrap();

        config.pending_admin = None;
        config.admin_timelock = None;

        env.storage().instance().set(&DataKey::AdminConfig, &config);

        env.events().publish(
            (Symbol::new(&env, "admin_rotation_cancelled"),),
            AdminRotationCancelled(config.current_admin, cancelled_admin),
        );
    }

    /// Get current admin configuration
    pub fn get_admin_config(env: Env) -> AdminConfig {
        env.storage().instance().get(&DataKey::AdminConfig)
            .unwrap_or_else(|| panic!("Contract not initialized"))
    }

    /// Check if caller is current admin
    pub fn is_admin(env: Env, caller: Address) -> bool {
        let config: AdminConfig = env.storage().instance().get(&DataKey::AdminConfig)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        caller == config.current_admin
    }
}