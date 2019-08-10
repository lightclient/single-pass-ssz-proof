#![cfg_attr(not(feature = "std"), no_std)]

pub mod update;
pub mod utils;

const CHUNK_SIZE: usize = 32;

#[cfg(not(test))]
mod native {
    extern "C" {
        pub fn eth2_loadPreStateRoot(offset: *const u32);
        pub fn eth2_blockDataSize() -> u32;
        pub fn eth2_blockDataCopy(outputOfset: *const u32, offset: u32, length: u32);
        pub fn eth2_savePostStateRoot(offset: *const u32);
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    // Get input size
    let input_size = unsafe { native::eth2_blockDataSize() as usize };

    // // Copy input into buffer
    let mut input = [0u8; 41 * CHUNK_SIZE];
    unsafe {
        native::eth2_blockDataCopy(input.as_mut_ptr() as *const u32, 0, input_size as u32);
    }

    // The new balance will `1u256`.
    let new_balance: [u8; CHUNK_SIZE] = {
        let mut tmp = [0u8; CHUNK_SIZE];
        tmp[0] = 1;
        tmp
    };

    // Buffers to hold the calculated roots
    let mut calculated_pre_state_root = [0u8; CHUNK_SIZE];
    let mut calculated_post_state_root = [0u8; CHUNK_SIZE];

    // Update branch in a single pass
    update::update(
        99999 + (1 << 40),
        &input,
        &mut calculated_pre_state_root,
        &mut calculated_post_state_root,
        &new_balance,
    );

    // Verify pre-state root == calculated pre-state root
    let mut pre_state_root = [0u8; CHUNK_SIZE];
    unsafe { native::eth2_loadPreStateRoot(pre_state_root.as_mut_ptr() as *const u32) }
    assert_eq!(pre_state_root, calculated_pre_state_root);

    // Return post state
    unsafe { native::eth2_savePostStateRoot(calculated_post_state_root.as_mut_ptr() as *const u32) }
}
