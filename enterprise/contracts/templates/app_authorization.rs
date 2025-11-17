#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Application Authorization Template for Enterprise E2 Multipass
///
/// This template implements app-aware authorization and scoped permissions,
/// integrated with the E2 Multipass application service.
///
/// Features:
/// - OAuth-like scopes for fine-grained permissions
/// - Application registration and authentication
/// - Time-limited authorization grants
/// - Delegation and sub-delegation
/// - Resource ownership and access control
/// - Integration with E2 Multipass identity and application services

#[ink::contract]
mod app_authorization {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;

    /// Permission scope
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Scope {
        /// Read user profile
        ReadProfile,
        /// Write user profile
        WriteProfile,
        /// Read wallet data
        ReadWallet,
        /// Execute wallet transactions
        ExecuteTransactions,
        /// Read asset balances
        ReadAssets,
        /// Transfer assets
        TransferAssets,
        /// Manage smart contracts
        ManageContracts,
        /// Administrative access
        Admin,
        /// Custom scope
        Custom(Vec<u8>),
    }

    /// Application registration
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Application {
        /// Application ID
        app_id: [u8; 32],
        /// Application name
        name: String,
        /// Owner identity
        owner: AccountId,
        /// Owner's E2 identity
        owner_identity: [u8; 32],
        /// Redirect URIs for OAuth flow
        redirect_uris: Vec<String>,
        /// Requested scopes
        requested_scopes: Vec<Scope>,
        /// Active status
        active: bool,
        /// Registration timestamp
        registered_at: Timestamp,
    }

    /// Authorization grant
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct AuthGrant {
        /// Grant ID
        grant_id: u64,
        /// Resource owner (user)
        resource_owner: AccountId,
        /// Resource owner's E2 identity
        owner_identity: [u8; 32],
        /// Application being authorized
        app_id: [u8; 32],
        /// Granted scopes
        scopes: Vec<Scope>,
        /// Grant expiry timestamp
        expires_at: Timestamp,
        /// Issued timestamp
        issued_at: Timestamp,
        /// Revoked status
        revoked: bool,
        /// Can be delegated
        delegatable: bool,
    }

    /// Delegated authorization
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Delegation {
        /// Delegation ID
        delegation_id: u64,
        /// Original grant ID
        source_grant_id: u64,
        /// Delegator
        delegator: AccountId,
        /// Delegate (who receives permission)
        delegate: AccountId,
        /// Delegate's E2 identity
        delegate_identity: [u8; 32],
        /// Delegated scopes (subset of original)
        scopes: Vec<Scope>,
        /// Expiry (must be <= source grant expiry)
        expires_at: Timestamp,
        /// Revoked status
        revoked: bool,
    }

    /// Resource ownership
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Resource {
        /// Resource ID
        resource_id: [u8; 32],
        /// Owner
        owner: AccountId,
        /// Resource type (contract, asset, data)
        resource_type: Vec<u8>,
        /// Required scopes for access
        required_scopes: Vec<Scope>,
        /// Public access allowed
        public_access: bool,
        /// Metadata hash
        metadata_hash: Option<[u8; 32]>,
    }

    /// Contract storage
    #[ink(storage)]
    pub struct AppAuthorization {
        /// Registered applications
        applications: Mapping<[u8; 32], Application>,
        /// Authorization grants (mapping: grant_id -> grant)
        grants: Mapping<u64, AuthGrant>,
        /// User's grants (mapping: user -> list of grant IDs)
        user_grants: Mapping<AccountId, Vec<u64>>,
        /// App's grants (mapping: app_id -> list of grant IDs)
        app_grants: Mapping<[u8; 32], Vec<u64>>,
        /// Delegations
        delegations: Mapping<u64, Delegation>,
        /// Resources
        resources: Mapping<[u8; 32], Resource>,
        /// Grant counter
        grant_count: u64,
        /// Delegation counter
        delegation_count: u64,
        /// Contract owner
        owner: AccountId,
        /// Default grant validity (milliseconds)
        default_grant_validity_ms: u64,
    }

    /// Events
    #[ink(event)]
    pub struct ApplicationRegistered {
        #[ink(topic)]
        app_id: [u8; 32],
        owner: AccountId,
        name: String,
    }

    #[ink(event)]
    pub struct GrantIssued {
        #[ink(topic)]
        grant_id: u64,
        #[ink(topic)]
        resource_owner: AccountId,
        #[ink(topic)]
        app_id: [u8; 32],
    }

    #[ink(event)]
    pub struct GrantRevoked {
        #[ink(topic)]
        grant_id: u64,
        #[ink(topic)]
        revoked_by: AccountId,
    }

    #[ink(event)]
    pub struct DelegationCreated {
        #[ink(topic)]
        delegation_id: u64,
        #[ink(topic)]
        delegator: AccountId,
        #[ink(topic)]
        delegate: AccountId,
    }

    #[ink(event)]
    pub struct ResourceRegistered {
        #[ink(topic)]
        resource_id: [u8; 32],
        #[ink(topic)]
        owner: AccountId,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Application not found
        ApplicationNotFound,
        /// Application already registered
        ApplicationAlreadyRegistered,
        /// Grant not found
        GrantNotFound,
        /// Grant expired
        GrantExpired,
        /// Grant revoked
        GrantRevoked,
        /// Insufficient scopes
        InsufficientScopes,
        /// Not authorized
        Unauthorized,
        /// Resource not found
        ResourceNotFound,
        /// Delegation not allowed
        DelegationNotAllowed,
        /// Invalid delegation
        InvalidDelegation,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl AppAuthorization {
        /// Create a new app authorization contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                applications: Mapping::default(),
                grants: Mapping::default(),
                user_grants: Mapping::default(),
                app_grants: Mapping::default(),
                delegations: Mapping::default(),
                resources: Mapping::default(),
                grant_count: 0,
                delegation_count: 0,
                owner: Self::env().caller(),
                default_grant_validity_ms: 30 * 24 * 60 * 60 * 1000, // 30 days
            }
        }

        /// Register a new application
        #[ink(message)]
        pub fn register_application(
            &mut self,
            app_id: [u8; 32],
            name: String,
            owner_identity: [u8; 32],
            redirect_uris: Vec<String>,
            requested_scopes: Vec<Scope>,
        ) -> Result<()> {
            if self.applications.contains(app_id) {
                return Err(Error::ApplicationAlreadyRegistered);
            }

            let caller = self.env().caller();

            let app = Application {
                app_id,
                name: name.clone(),
                owner: caller,
                owner_identity,
                redirect_uris,
                requested_scopes,
                active: true,
                registered_at: self.env().block_timestamp(),
            };

            self.applications.insert(app_id, &app);

            self.env().emit_event(ApplicationRegistered {
                app_id,
                owner: caller,
                name,
            });

            Ok(())
        }

        /// Issue an authorization grant to an application
        #[ink(message)]
        pub fn issue_grant(
            &mut self,
            app_id: [u8; 32],
            owner_identity: [u8; 32],
            scopes: Vec<Scope>,
            validity_ms: Option<u64>,
            delegatable: bool,
        ) -> Result<u64> {
            // Verify application exists
            let _app = self.applications.get(app_id)
                .ok_or(Error::ApplicationNotFound)?;

            let caller = self.env().caller();
            let grant_id = self.grant_count;
            self.grant_count += 1;

            let now = self.env().block_timestamp();
            let expires_at = now + validity_ms.unwrap_or(self.default_grant_validity_ms);

            let grant = AuthGrant {
                grant_id,
                resource_owner: caller,
                owner_identity,
                app_id,
                scopes,
                expires_at,
                issued_at: now,
                revoked: false,
                delegatable,
            };

            self.grants.insert(grant_id, &grant);

            // Track user's grants
            let mut user_grant_list = self.user_grants.get(caller).unwrap_or_default();
            user_grant_list.push(grant_id);
            self.user_grants.insert(caller, &user_grant_list);

            // Track app's grants
            let mut app_grant_list = self.app_grants.get(app_id).unwrap_or_default();
            app_grant_list.push(grant_id);
            self.app_grants.insert(app_id, &app_grant_list);

            self.env().emit_event(GrantIssued {
                grant_id,
                resource_owner: caller,
                app_id,
            });

            Ok(grant_id)
        }

        /// Revoke an authorization grant
        #[ink(message)]
        pub fn revoke_grant(&mut self, grant_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut grant = self.grants.get(grant_id)
                .ok_or(Error::GrantNotFound)?;

            // Only resource owner can revoke
            if grant.resource_owner != caller {
                return Err(Error::Unauthorized);
            }

            grant.revoked = true;
            self.grants.insert(grant_id, &grant);

            self.env().emit_event(GrantRevoked {
                grant_id,
                revoked_by: caller,
            });

            Ok(())
        }

        /// Create a delegation from an existing grant
        #[ink(message)]
        pub fn create_delegation(
            &mut self,
            source_grant_id: u64,
            delegate: AccountId,
            delegate_identity: [u8; 32],
            scopes: Vec<Scope>,
            validity_ms: Option<u64>,
        ) -> Result<u64> {
            let caller = self.env().caller();
            let source_grant = self.grants.get(source_grant_id)
                .ok_or(Error::GrantNotFound)?;

            // Verify caller owns the grant
            if source_grant.resource_owner != caller {
                return Err(Error::Unauthorized);
            }

            // Verify grant is delegatable
            if !source_grant.delegatable {
                return Err(Error::DelegationNotAllowed);
            }

            // Verify scopes are subset of source grant
            for scope in &scopes {
                if !source_grant.scopes.contains(scope) {
                    return Err(Error::InsufficientScopes);
                }
            }

            let delegation_id = self.delegation_count;
            self.delegation_count += 1;

            let now = self.env().block_timestamp();
            let max_expiry = source_grant.expires_at;
            let requested_expiry = now + validity_ms.unwrap_or(self.default_grant_validity_ms);
            let expires_at = if requested_expiry > max_expiry {
                max_expiry
            } else {
                requested_expiry
            };

            let delegation = Delegation {
                delegation_id,
                source_grant_id,
                delegator: caller,
                delegate,
                delegate_identity,
                scopes,
                expires_at,
                revoked: false,
            };

            self.delegations.insert(delegation_id, &delegation);

            self.env().emit_event(DelegationCreated {
                delegation_id,
                delegator: caller,
                delegate,
            });

            Ok(delegation_id)
        }

        /// Check if account has required scopes for an app
        #[ink(message)]
        pub fn has_scopes(
            &self,
            account: AccountId,
            app_id: [u8; 32],
            required_scopes: Vec<Scope>,
        ) -> bool {
            // Get all grants for this app
            if let Some(grant_ids) = self.app_grants.get(app_id) {
                for grant_id in grant_ids {
                    if let Some(grant) = self.grants.get(grant_id) {
                        // Check if grant is valid
                        if grant.resource_owner == account
                            && !grant.revoked
                            && self.env().block_timestamp() <= grant.expires_at
                        {
                            // Check if all required scopes are present
                            let has_all = required_scopes.iter()
                                .all(|scope| grant.scopes.contains(scope));

                            if has_all {
                                return true;
                            }
                        }
                    }
                }
            }

            false
        }

        /// Register a resource for access control
        #[ink(message)]
        pub fn register_resource(
            &mut self,
            resource_id: [u8; 32],
            resource_type: Vec<u8>,
            required_scopes: Vec<Scope>,
            public_access: bool,
            metadata_hash: Option<[u8; 32]>,
        ) -> Result<()> {
            let caller = self.env().caller();

            let resource = Resource {
                resource_id,
                owner: caller,
                resource_type,
                required_scopes,
                public_access,
                metadata_hash,
            };

            self.resources.insert(resource_id, &resource);

            self.env().emit_event(ResourceRegistered {
                resource_id,
                owner: caller,
            });

            Ok(())
        }

        /// Check if account can access a resource
        #[ink(message)]
        pub fn can_access_resource(
            &self,
            account: AccountId,
            resource_id: [u8; 32],
            app_id: [u8; 32],
        ) -> bool {
            let resource = match self.resources.get(resource_id) {
                Some(r) => r,
                None => return false,
            };

            // Owner always has access
            if resource.owner == account {
                return true;
            }

            // Public resources allow anyone
            if resource.public_access {
                return true;
            }

            // Check if app has required scopes
            self.has_scopes(account, app_id, resource.required_scopes)
        }

        /// Get application details
        #[ink(message)]
        pub fn get_application(&self, app_id: [u8; 32]) -> Option<Application> {
            self.applications.get(app_id)
        }

        /// Get grant details
        #[ink(message)]
        pub fn get_grant(&self, grant_id: u64) -> Option<AuthGrant> {
            self.grants.get(grant_id)
        }

        /// Get user's grants
        #[ink(message)]
        pub fn get_user_grants(&self, account: AccountId) -> Vec<u64> {
            self.user_grants.get(account).unwrap_or_default()
        }

        /// Get total grants issued
        #[ink(message)]
        pub fn total_grants(&self) -> u64 {
            self.grant_count
        }
    }

    // === Unit Tests ===

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let auth = AppAuthorization::new();
            assert_eq!(auth.total_grants(), 0);
        }

        #[ink::test]
        fn register_application_works() {
            let mut auth = AppAuthorization::new();
            let app_id = [1u8; 32];

            let result = auth.register_application(
                app_id,
                String::from("Test App"),
                [0u8; 32],
                vec![],
                vec![Scope::ReadProfile],
            );

            assert!(result.is_ok());
        }

        #[ink::test]
        fn issue_grant_works() {
            let mut auth = AppAuthorization::new();
            let app_id = [1u8; 32];

            // Register app first
            auth.register_application(
                app_id,
                String::from("Test App"),
                [0u8; 32],
                vec![],
                vec![Scope::ReadProfile],
            )
            .unwrap();

            // Issue grant
            let grant_id = auth
                .issue_grant(app_id, [0u8; 32], vec![Scope::ReadProfile], None, false)
                .unwrap();

            assert_eq!(grant_id, 0);
            assert_eq!(auth.total_grants(), 1);
        }
    }
}
