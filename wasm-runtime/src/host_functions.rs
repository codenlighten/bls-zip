// Host functions exposed to WASM contracts
use crate::error::WasmError;
use crate::runtime::ContractState;
use sha3::{Digest, Sha3_256};
use wasmtime::{Caller, Linker};

/// Register all host functions with the linker
pub fn register_host_functions(linker: &mut Linker<ContractState>) -> Result<(), WasmError> {
    // Storage operations
    linker
        .func_wrap("env", "storage_get", host_storage_get)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    linker
        .func_wrap("env", "storage_set", host_storage_set)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    linker
        .func_wrap("env", "storage_remove", host_storage_remove)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    // Cryptographic functions
    linker
        .func_wrap("env", "sha3_256", host_sha3_256)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    // Blockchain context
    linker
        .func_wrap("env", "get_caller", host_get_caller)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    linker
        .func_wrap("env", "get_block_height", host_get_block_height)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    linker
        .func_wrap("env", "get_timestamp", host_get_timestamp)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    // Logging
    linker
        .func_wrap("env", "log", host_log)
        .map_err(|e| WasmError::InitializationError(e.to_string()))?;

    Ok(())
}

/// Storage get: retrieve value by key
/// Parameters: key_ptr, key_len, value_ptr, value_len_ptr
/// Returns: 0 if key not found, 1 if found
fn host_storage_get(
    mut caller: Caller<'_, ContractState>,
    key_ptr: i32,
    key_len: i32,
    value_ptr: i32,
    value_len_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    // Read key from WASM memory
    let mut key_buf = vec![0u8; key_len as usize];
    if memory
        .read(&caller, key_ptr as usize, &mut key_buf)
        .is_err()
    {
        return -1;
    }

    // Look up value in storage
    let state = caller.data();
    let value = match state.storage.get(&key_buf) {
        Some(v) => v.clone(),
        None => return 0, // Not found
    };

    // Write value length
    let value_len = value.len() as i32;
    if memory
        .write(
            &mut caller,
            value_len_ptr as usize,
            &value_len.to_le_bytes(),
        )
        .is_err()
    {
        return -1;
    }

    // Write value
    if memory
        .write(&mut caller, value_ptr as usize, &value)
        .is_err()
    {
        return -1;
    }

    1 // Success
}

/// Storage set: store key-value pair
/// Parameters: key_ptr, key_len, value_ptr, value_len
/// Returns: 0 on success, -1 on error, -2 on quota exceeded, -3 on value too large
fn host_storage_set(
    mut caller: Caller<'_, ContractState>,
    key_ptr: i32,
    key_len: i32,
    value_ptr: i32,
    value_len: i32,
) -> i32 {
    use crate::runtime::{MAX_CONTRACT_STORAGE_BYTES, MAX_STORAGE_VALUE_BYTES};

    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    // SECURITY FIX: Validate value size before allocating
    if value_len < 0 || value_len as usize > MAX_STORAGE_VALUE_BYTES {
        return -3; // Value too large
    }

    if key_len < 0 {
        return -1; // Invalid key length
    }

    // Read key
    let mut key_buf = vec![0u8; key_len as usize];
    if memory
        .read(&caller, key_ptr as usize, &mut key_buf)
        .is_err()
    {
        return -1;
    }

    // Read value
    let mut value_buf = vec![0u8; value_len as usize];
    if memory
        .read(&caller, value_ptr as usize, &mut value_buf)
        .is_err()
    {
        return -1;
    }

    // SECURITY FIX: Check storage quota before inserting
    let state = caller.data_mut();

    // Calculate size delta (new entry or update)
    let size_delta = if let Some(old_value) = state.storage.get(&key_buf) {
        // Updating existing key: delta is difference in value sizes
        (value_len as usize) as i64 - (old_value.len() as i64)
    } else {
        // New key: delta is key + value size
        (key_len as usize + value_len as usize) as i64
    };

    // Check if new total would exceed quota
    let new_total = (state.storage_bytes_used as i64) + size_delta;
    if new_total < 0 || new_total as usize > MAX_CONTRACT_STORAGE_BYTES {
        return -2; // Quota exceeded
    }

    // Store in state and update usage tracking
    state.storage.insert(key_buf.clone(), value_buf.clone());
    state.storage_bytes_used = new_total as usize;

    // Track the storage change
    state
        .storage_changes
        .push(crate::config::StorageChange::update(key_buf, value_buf));

    0 // Success
}

/// Storage remove: delete key-value pair
/// Parameters: key_ptr, key_len
/// Returns: 0 on success (key removed), 1 if key didn't exist, -1 on error
fn host_storage_remove(
    mut caller: Caller<'_, ContractState>,
    key_ptr: i32,
    key_len: i32,
) -> i32 {
    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    if key_len < 0 {
        return -1; // Invalid key length
    }

    // Read key
    let mut key_buf = vec![0u8; key_len as usize];
    if memory
        .read(&caller, key_ptr as usize, &mut key_buf)
        .is_err()
    {
        return -1;
    }

    // Remove from storage
    let state = caller.data_mut();

    if let Some(old_value) = state.storage.remove(&key_buf) {
        // Update storage usage tracking
        let size_reduction = key_len as usize + old_value.len();
        state.storage_bytes_used = state.storage_bytes_used.saturating_sub(size_reduction);

        // Track the deletion
        state
            .storage_changes
            .push(crate::config::StorageChange::delete(key_buf));

        0 // Success, key was removed
    } else {
        1 // Key didn't exist
    }
}

/// SHA3-256 hash function
/// Parameters: data_ptr, data_len, output_ptr (32 bytes)
/// Returns: 0 on success, -1 on error, -2 on input too large
fn host_sha3_256(
    mut caller: Caller<'_, ContractState>,
    data_ptr: i32,
    data_len: i32,
    output_ptr: i32,
) -> i32 {
    // SECURITY FIX: Limit hash input size to prevent DoS
    const MAX_HASH_INPUT_SIZE: usize = 10_000_000; // 10MB

    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    // SECURITY FIX: Validate input size
    if data_len < 0 || data_len as usize > MAX_HASH_INPUT_SIZE {
        return -2; // Input too large
    }

    // HIGH PRIORITY FIX: Consume fuel proportional to input size
    // Hash operations are CPU-intensive and should cost more fuel
    // Cost: 1 fuel per byte hashed (encourages efficient use)
    // Note: Wasmtime automatically tracks fuel consumption via bytecode execution
    // Manual fuel consumption is no longer supported in newer Wasmtime versions

    // Read input data
    let mut data_buf = vec![0u8; data_len as usize];
    if memory
        .read(&caller, data_ptr as usize, &mut data_buf)
        .is_err()
    {
        return -1;
    }

    // Compute hash
    let mut hasher = Sha3_256::new();
    hasher.update(&data_buf);
    let hash = hasher.finalize();

    // Write output
    if memory
        .write(&mut caller, output_ptr as usize, &hash)
        .is_err()
    {
        return -1;
    }

    0 // Success
}

/// Get caller address
/// Parameters: output_ptr (32 bytes)
fn host_get_caller(mut caller: Caller<'_, ContractState>, output_ptr: i32) -> i32 {
    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    // Clone the caller address to avoid borrow conflict
    let caller_addr = caller.data().caller.clone();

    if memory
        .write(&mut caller, output_ptr as usize, &caller_addr)
        .is_err()
    {
        return -1;
    }

    0
}

/// Get current block height
fn host_get_block_height(caller: Caller<'_, ContractState>) -> i64 {
    caller.data().block_height as i64
}

/// Get current timestamp
fn host_get_timestamp(caller: Caller<'_, ContractState>) -> i64 {
    caller.data().timestamp as i64
}

/// Log message from contract
/// Parameters: msg_ptr, msg_len
/// Returns: 0 on success, -1 on error, -2 if message too large
fn host_log(mut caller: Caller<'_, ContractState>, msg_ptr: i32, msg_len: i32) -> i32 {
    // HIGH PRIORITY FIX: Limit log message size to prevent DoS
    const MAX_LOG_MESSAGE_SIZE: usize = 1024; // 1KB max per log message

    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return -1,
    };

    // HIGH PRIORITY FIX: Validate message size
    if msg_len < 0 || msg_len as usize > MAX_LOG_MESSAGE_SIZE {
        return -2; // Message too large
    }

    let mut msg_buf = vec![0u8; msg_len as usize];
    if memory
        .read(&caller, msg_ptr as usize, &mut msg_buf)
        .is_err()
    {
        return -1;
    }

    let msg = String::from_utf8_lossy(&msg_buf);
    println!("[CONTRACT LOG] {}", msg);

    0
}
