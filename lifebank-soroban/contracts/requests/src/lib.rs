#![no_std]

mod error;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use crate::error::ContractError;
pub use crate::types::{
    BloodComponent, BloodRequest, BloodType, ContractMetadata, DataKey, RequestCreatedEvent,
    RequestStatus, Urgency,
};

mod validation;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct RequestContract;

#[contractimpl]
impl RequestContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        inventory_contract: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        if storage::is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }

        storage::set_admin(&env, &admin);
        storage::set_inventory_contract(&env, &inventory_contract);
        storage::set_request_counter(&env, 0);
        storage::set_metadata(&env, &storage::default_metadata(&env));
        storage::authorize_hospital(&env, &admin);
        storage::set_initialized(&env);

        events::emit_initialized(&env, &admin, &inventory_contract);

        Ok(())
    }

    pub fn authorize_hospital(env: Env, hospital: Address) -> Result<(), ContractError> {
        storage::require_initialized(&env)?;
        storage::get_admin(&env).require_auth();
        storage::authorize_hospital(&env, &hospital);
        Ok(())
    }

    pub fn revoke_hospital(env: Env, hospital: Address) -> Result<(), ContractError> {
        storage::require_initialized(&env)?;
        storage::get_admin(&env).require_auth();
        storage::revoke_hospital(&env, &hospital);
        Ok(())
    }

    pub fn authorize_blood_bank(env: Env, blood_bank: Address) -> Result<(), ContractError> {
        storage::require_initialized(&env)?;
        storage::get_admin(&env).require_auth();
        storage::authorize_blood_bank(&env, &blood_bank);
        Ok(())
    }

    pub fn revoke_blood_bank(env: Env, blood_bank: Address) -> Result<(), ContractError> {
        storage::require_initialized(&env)?;
        storage::get_admin(&env).require_auth();
        storage::revoke_blood_bank(&env, &blood_bank);
        Ok(())
    }

    pub fn authorize_rider(env: Env, rider: Address) -> Result<(), ContractError> {
        storage::require_initialized(&env)?;
        storage::get_admin(&env).require_auth();
        storage::authorize_rider(&env, &rider);
        Ok(())
    }

    pub fn revoke_rider(env: Env, rider: Address) -> Result<(), ContractError> {
        storage::require_initialized(&env)?;
        storage::get_admin(&env).require_auth();
        storage::revoke_rider(&env, &rider);
        Ok(())
    }

    pub fn create_request(
        env: Env,
        hospital: Address,
        blood_type: BloodType,
        component: BloodComponent,
        quantity_ml: u32,
        urgency: Urgency,
        required_by_timestamp: u64,
    ) -> Result<u64, ContractError> {
        hospital.require_auth();
        storage::require_initialized(&env)?;

        if !storage::is_hospital_authorized(&env, &hospital) {
            return Err(ContractError::NotAuthorizedHospital);
        }

        validation::validate_timestamp(&env, required_by_timestamp)?;
        validation::validate_quantity(quantity_ml)?;

        let request_id = storage::increment_request_counter(&env);
        let request = BloodRequest {
            id: request_id,
            hospital_id: hospital.clone(),
            blood_type,
            component,
            quantity_ml,
            urgency,
            created_timestamp: env.ledger().timestamp(),
            required_by_timestamp,
            status: RequestStatus::Pending,
            assigned_units: soroban_sdk::Vec::new(&env),
            fulfilled_quantity_ml: 0,
        };

        storage::set_request(&env, &request);
        events::emit_request_created(&env, &request);

        Ok(request_id)
    }

    pub fn get_request(env: Env, request_id: u64) -> Result<BloodRequest, ContractError> {
        storage::require_initialized(&env)?;
        storage::get_request(&env, request_id).ok_or(ContractError::RequestNotFound)
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        storage::require_initialized(&env)?;
        Ok(storage::get_admin(&env))
    }

    pub fn get_inventory_contract(env: Env) -> Result<Address, ContractError> {
        storage::require_initialized(&env)?;
        Ok(storage::get_inventory_contract(&env))
    }

    pub fn get_request_counter(env: Env) -> Result<u64, ContractError> {
        storage::require_initialized(&env)?;
        Ok(storage::get_request_counter(&env))
    }

    pub fn get_metadata(env: Env) -> Result<ContractMetadata, ContractError> {
        storage::require_initialized(&env)?;
        Ok(storage::get_metadata(&env))
    }

    pub fn is_hospital_authorized(env: Env, hospital: Address) -> bool {
        storage::is_hospital_authorized(&env, &hospital)
    }

    pub fn is_initialized(env: Env) -> bool {
        storage::is_initialized(&env)
    }

    /// Update request status with role-based access control
    ///
    /// Only specific roles can perform specific status transitions:
    /// - BloodBank: Pending → InProgress (marking request as being processed)
    /// - Rider: InProgress → InTransit (marking request as in delivery)
    /// - Hospital: InTransit → Fulfilled (confirming delivery received)
    /// - Hospital: Pending → Cancelled (cancelling their own request)
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `caller` - Address performing the status update (must be authenticated)
    /// * `request_id` - ID of the request to update
    /// * `new_status` - New status to set
    ///
    /// # Errors
    /// - `RequestNotFound`: Request with given ID doesn't exist
    /// - `UnauthorizedStatusTransition`: Caller's role doesn't allow this transition
    /// - `InvalidStatusTransition`: The status transition itself is not valid
    pub fn update_request_status(
        env: Env,
        caller: Address,
        request_id: u64,
        new_status: RequestStatus,
    ) -> Result<BloodRequest, ContractError> {
        caller.require_auth();
        storage::require_initialized(&env)?;

        let mut request = storage::get_request(&env, request_id)
            .ok_or(ContractError::RequestNotFound)?;

        let current_status = request.status;

        // Determine caller's role
        let caller_role = if storage::is_hospital_authorized(&env, &caller) {
            types::Role::Hospital
        } else if storage::is_blood_bank_authorized(&env, &caller) {
            types::Role::BloodBank
        } else if storage::is_rider_authorized(&env, &caller) {
            types::Role::Rider
        } else {
            return Err(ContractError::Unauthorized);
        };

        // Validate role-based status transitions
        use types::{RequestStatus::*, Role};
        let is_authorized = matches!(
            (caller_role, &current_status, &new_status),
            (Role::BloodBank, Pending, Approved)
                | (Role::Rider, Approved, Fulfilled)
                | (Role::Hospital, Fulfilled, Fulfilled)
                | (Role::Hospital, Pending, Cancelled)
        );

        if !is_authorized {
            return Err(ContractError::UnauthorizedStatusTransition);
        }

        // Update the request
        request.status = new_status;
        storage::set_request(&env, &request);

        events::emit_status_updated(&env, request_id, current_status, new_status, &caller);

        Ok(request)
    }

    pub fn is_blood_bank_authorized(env: Env, blood_bank: Address) -> bool {
        storage::is_blood_bank_authorized(&env, &blood_bank)
    }

    pub fn is_rider_authorized(env: Env, rider: Address) -> bool {
        storage::is_rider_authorized(&env, &rider)
    }
}
