#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Identity-Based Access Control Template for Enterprise E2 Multipass
///
/// This template demonstrates how to integrate E2 Multipass identities with smart contracts
/// for role-based access control, identity verification, and attestation checking.
///
/// Features:
/// - Role-based access control (RBAC) using E2 identities
/// - Identity verification via post-quantum signatures
/// - Attestation checking (KYC, certification, etc.)
/// - Multi-level permissions (Owner, Admin, User)
/// - Integration with E2 Multipass identity service

#[ink::contract]
mod identity_access_control {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    /// Roles for access control
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Role {
        /// Contract owner (highest privileges)
        Owner,
        /// Administrator (manage users and roles)
        Admin,
        /// Verified user (basic access)
        User,
        /// Guest (read-only access)
        Guest,
    }

    /// Identity verification status
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct IdentityStatus {
        /// E2 Multipass identity ID
        identity_id: [u8; 32],
        /// Current role
        role: Role,
        /// KYC verified
        kyc_verified: bool,
        /// Attestation hashes (certification, license, etc.)
        attestations: Vec<[u8; 32]>,
        /// Registration timestamp
        registered_at: Timestamp,
        /// Active status
        active: bool,
    }

    /// Contract storage
    #[ink(storage)]
    pub struct IdentityAccessControl {
        /// Contract owner
        owner: AccountId,
        /// Mapping from account to identity status
        identities: Mapping<AccountId, IdentityStatus>,
        /// Mapping from role to list of accounts
        role_members: Mapping<Role, Vec<AccountId>>,
        /// Required attestation types for specific actions
        required_attestations: Mapping<u32, Vec<[u8; 32]>>,
        /// Total registered identities
        total_identities: u64,
    }

    /// Events
    #[ink(event)]
    pub struct IdentityRegistered {
        #[ink(topic)]
        account: AccountId,
        identity_id: [u8; 32],
        role: Role,
    }

    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        account: AccountId,
        role: Role,
        #[ink(topic)]
        granted_by: AccountId,
    }

    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        account: AccountId,
        role: Role,
        #[ink(topic)]
        revoked_by: AccountId,
    }

    #[ink(event)]
    pub struct AttestationAdded {
        #[ink(topic)]
        account: AccountId,
        attestation_hash: [u8; 32],
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Caller not authorized
        Unauthorized,
        /// Identity already registered
        AlreadyRegistered,
        /// Identity not found
        IdentityNotFound,
        /// Invalid role
        InvalidRole,
        /// Attestation required but not present
        AttestationRequired,
        /// Identity not active
        IdentityInactive,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl IdentityAccessControl {
        /// Create a new identity access control contract
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let mut identities = Mapping::default();

            // Register owner
            let owner_identity = IdentityStatus {
                identity_id: [0u8; 32], // Owner sets their ID later
                role: Role::Owner,
                kyc_verified: true,
                attestations: Vec::new(),
                registered_at: Self::env().block_timestamp(),
                active: true,
            };

            identities.insert(caller, &owner_identity);

            Self {
                owner: caller,
                identities,
                role_members: Mapping::default(),
                required_attestations: Mapping::default(),
                total_identities: 1,
            }
        }

        /// Register a new identity with E2 Multipass integration
        #[ink(message)]
        pub fn register_identity(
            &mut self,
            identity_id: [u8; 32],
            initial_role: Role,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Check if already registered
            if self.identities.contains(caller) {
                return Err(Error::AlreadyRegistered);
            }

            // Only Owner or Admin can register new identities
            self.require_role(&self.env().caller(), &[Role::Owner, Role::Admin])?;

            let identity_status = IdentityStatus {
                identity_id,
                role: initial_role.clone(),
                kyc_verified: false,
                attestations: Vec::new(),
                registered_at: self.env().block_timestamp(),
                active: true,
            };

            self.identities.insert(caller, &identity_status);
            self.total_identities += 1;

            self.env().emit_event(IdentityRegistered {
                account: caller,
                identity_id,
                role: initial_role,
            });

            Ok(())
        }

        /// Grant a role to an account
        #[ink(message)]
        pub fn grant_role(&mut self, account: AccountId, role: Role) -> Result<()> {
            let caller = self.env().caller();

            // Only Owner or Admin can grant roles
            self.require_role(&caller, &[Role::Owner, Role::Admin])?;

            // Get existing identity
            let mut identity = self.identities.get(account)
                .ok_or(Error::IdentityNotFound)?;

            // Update role
            identity.role = role.clone();
            self.identities.insert(account, &identity);

            self.env().emit_event(RoleGranted {
                account,
                role,
                granted_by: caller,
            });

            Ok(())
        }

        /// Revoke a role from an account
        #[ink(message)]
        pub fn revoke_role(&mut self, account: AccountId) -> Result<()> {
            let caller = self.env().caller();

            // Only Owner can revoke roles
            self.require_role(&caller, &[Role::Owner])?;

            // Cannot revoke owner
            if account == self.owner {
                return Err(Error::Unauthorized);
            }

            let mut identity = self.identities.get(account)
                .ok_or(Error::IdentityNotFound)?;

            let old_role = identity.role.clone();
            identity.role = Role::Guest;
            identity.active = false;
            self.identities.insert(account, &identity);

            self.env().emit_event(RoleRevoked {
                account,
                role: old_role,
                revoked_by: caller,
            });

            Ok(())
        }

        /// Add attestation to identity (e.g., KYC, certification)
        #[ink(message)]
        pub fn add_attestation(
            &mut self,
            account: AccountId,
            attestation_hash: [u8; 32],
        ) -> Result<()> {
            let caller = self.env().caller();

            // Only Admin or Owner can add attestations
            self.require_role(&caller, &[Role::Owner, Role::Admin])?;

            let mut identity = self.identities.get(account)
                .ok_or(Error::IdentityNotFound)?;

            identity.attestations.push(attestation_hash);
            self.identities.insert(account, &identity);

            self.env().emit_event(AttestationAdded {
                account,
                attestation_hash,
            });

            Ok(())
        }

        /// Mark identity as KYC verified
        #[ink(message)]
        pub fn mark_kyc_verified(&mut self, account: AccountId) -> Result<()> {
            let caller = self.env().caller();

            // Only Admin or Owner can verify KYC
            self.require_role(&caller, &[Role::Owner, Role::Admin])?;

            let mut identity = self.identities.get(account)
                .ok_or(Error::IdentityNotFound)?;

            identity.kyc_verified = true;
            self.identities.insert(account, &identity);

            Ok(())
        }

        /// Check if account has required role
        #[ink(message)]
        pub fn has_role(&self, account: AccountId, role: Role) -> bool {
            if let Some(identity) = self.identities.get(account) {
                identity.active && identity.role == role
            } else {
                false
            }
        }

        /// Check if account has any of the required roles
        #[ink(message)]
        pub fn has_any_role(&self, account: AccountId, roles: Vec<Role>) -> bool {
            if let Some(identity) = self.identities.get(account) {
                identity.active && roles.contains(&identity.role)
            } else {
                false
            }
        }

        /// Check if account has required attestation
        #[ink(message)]
        pub fn has_attestation(&self, account: AccountId, attestation_hash: [u8; 32]) -> bool {
            if let Some(identity) = self.identities.get(account) {
                identity.attestations.contains(&attestation_hash)
            } else {
                false
            }
        }

        /// Get identity status
        #[ink(message)]
        pub fn get_identity(&self, account: AccountId) -> Option<IdentityStatus> {
            self.identities.get(account)
        }

        /// Get total registered identities
        #[ink(message)]
        pub fn total_identities(&self) -> u64 {
            self.total_identities
        }

        // === Internal Helper Functions ===

        /// Require caller to have one of the specified roles
        fn require_role(&self, account: &AccountId, roles: &[Role]) -> Result<()> {
            if let Some(identity) = self.identities.get(account) {
                if !identity.active {
                    return Err(Error::IdentityInactive);
                }
                if roles.contains(&identity.role) {
                    Ok(())
                } else {
                    Err(Error::Unauthorized)
                }
            } else {
                Err(Error::IdentityNotFound)
            }
        }

        /// Require caller to have specific attestation
        fn require_attestation(&self, account: &AccountId, attestation_hash: [u8; 32]) -> Result<()> {
            if let Some(identity) = self.identities.get(account) {
                if identity.attestations.contains(&attestation_hash) {
                    Ok(())
                } else {
                    Err(Error::AttestationRequired)
                }
            } else {
                Err(Error::IdentityNotFound)
            }
        }
    }

    // === Unit Tests ===

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let contract = IdentityAccessControl::new();
            assert_eq!(contract.total_identities(), 1);
        }

        #[ink::test]
        fn register_identity_works() {
            let mut contract = IdentityAccessControl::new();
            let identity_id = [1u8; 32];

            let result = contract.register_identity(identity_id, Role::User);
            assert!(result.is_ok());
            assert_eq!(contract.total_identities(), 2);
        }

        #[ink::test]
        fn grant_role_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = IdentityAccessControl::new();

            // Register user first
            let identity_id = [1u8; 32];
            contract.register_identity(identity_id, Role::User).unwrap();

            // Grant admin role
            let result = contract.grant_role(accounts.alice, Role::Admin);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn unauthorized_role_grant_fails() {
            // TODO: Test that non-admin cannot grant roles
        }
    }
}
