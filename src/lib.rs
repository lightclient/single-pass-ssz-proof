#![cfg_attr(not(feature = "std"), no_std)]

pub mod utils;
use utils::hash;

const TREE_SIZE: usize = 4;
const NUM_HASHES: usize = TREE_SIZE + 1;

// A few caveats at the moment:
//
// i) Only supports a single branch.
pub fn single_pass(
    mut index: usize,
    chunks: &[&mut [u8; 32]; NUM_HASHES],
    calculated_pre_state_root: &mut [u8; 32],
    calculated_post_state_root: &mut [u8; 32],
    new_balance: &[u8; 32],
) {
    // Initialize buffers to hold the different calculations. The first 32 bytes are the left
    // chunk and the second 32 bytes is the right chunk.
    let mut pre_buf = [0u8; 64];
    let mut post_buf = [0u8; 64];

    // Copy the starting bytes for both buffers. They will be more into the correct slot later
    // depending on the parity of the leaf.
    pre_buf[0..32].copy_from_slice(chunks[0]);
    post_buf[0..32].copy_from_slice(new_balance);

    // Begin calculating the root -- skip the first chunk since it has already been loaded into the
    // buffer.
    for chunk in chunks.iter().skip(1) {
        // The leaf's parity determines which slot the chunk data should go.
        // Even => Left node  => 0..32
        // Odd  => Right node => 32..64
        let parity = index % 2;

        // If the last calculated hash was for an odd node, it should be in the second 32 bytes of
        // the hash buffer.
        if parity == 1 {
            let mut tmp = [0u8; 32];
            tmp.copy_from_slice(&pre_buf[0..32]);
            pre_buf[32..64].copy_from_slice(&tmp);

            tmp.copy_from_slice(&post_buf[0..32]);
            post_buf[32..64].copy_from_slice(&tmp);
        }

        // Copy the sibling chunk into the buffer to be hashed. Xor the leaf's parity to get the
        // sibling's parity.
        let begin = 32 * (parity ^ 1);
        let end = 32 * (parity ^ 1) + 32;
        pre_buf[begin..end].copy_from_slice(*chunk);
        post_buf[begin..end].copy_from_slice(*chunk);

        #[cfg(feature = "std")]
        {
            let left = index & (-2isize as usize);
            let right = left + 1;
            println!(
                "last calculated: {}\nh({}, {})\npre-state:  {} | {}\npost-state: {} | {}\n",
                index,
                left,
                right,
                hex::encode(&pre_buf[0..32]),
                hex::encode(&pre_buf[32..64]),
                hex::encode(&post_buf[0..32]),
                hex::encode(&post_buf[32..64]),
            );
        }

        // The hash function will hash all 64 bytes & replace the first 32 bytes with the new hash.
        hash(&mut pre_buf);
        hash(&mut post_buf);

        index = index / 2;
    }

    // Return the calculated post-state root.
    calculated_pre_state_root.copy_from_slice(&pre_buf[0..32]);
    calculated_post_state_root.copy_from_slice(&post_buf[0..32]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::generate_branch;

    #[test]
    fn sanity_check() {
        // For this example, we'll assume we're operating on an object with the following
        // structure:
        //
        // AccountBalance => FixedVector[u256, 16];
        //
        // All account's start with a balance of 0. Therefore, the accounts' root before updating
        // is `zh(4)`.
        //
        //                 +-------- 1 --------+                <= zh(4)
        //                /                     \
        //           +-- 2 --+               +-- 3 --+          <= zh(3)
        //          /         \             /         \
        //         4           5           6           7        <= zh(2)
        //       /   \       /   \       /   \       /  \
        //      8     9     10   11     12   13     14   15     <= zh(1)
        //     / \   / \   / \   / \   / \   / \   / \   / \
        //    16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31   <= zh(0) <= account balance
        //     | |   | |   | |   | |   | |   | |   | |   | |
        //     v v   v v   v v   v v   v v   v v   v v   v v
        //
        //     0 1   2 3   4 5   6 7   8 9  10 11 12 13 14 15   <= account index

        // The account which the proof is for and whose balance will be updated
        let account: usize = 0;

        let mut input = [
            &mut [0u8; 32],
            &mut [0u8; 32],
            &mut [0u8; 32],
            &mut [0u8; 32],
            &mut [0u8; 32],
        ];

        // Generate a merkle proof for an account with a balance of 0. Since all account have a
        // balance of 0, this initial proof can be used interchangably amongst them.
        generate_branch(4, &mut input[0..5]);

        // The new balance will `1u256`.
        let new_balance: [u8; 32] = {
            let mut tmp = [0u8; 32];
            tmp[0] = 1;
            tmp
        };

        // Buffer to hold the calculated post-state root
        let mut calculated_pre_state_root = [0u8; 32];
        let mut calculated_post_state_root = [0u8; 32];

        // Calculate the post-state root and verify the pre-state root in one pass.
        single_pass(
            account + 16,
            &input,
            &mut calculated_pre_state_root,
            &mut calculated_post_state_root,
            &new_balance,
        );

        #[cfg(feature = "std")]
        println!(
            "final:\npre-state root:  {}\npost-state root: {}",
            hex::encode(&calculated_pre_state_root),
            hex::encode(&calculated_post_state_root)
        );

        // Verify that the calculated root is equal to the expected roots.
        let pre_state_root =
            &hex::decode("536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c")
                .unwrap()[0..32];
        let post_state_root =
            &hex::decode("0eefb94faea1cefdd28c895a51ba5822bcac513cd59413ece941e21d78bc83c4")
                .unwrap()[0..32];

        assert_eq!(pre_state_root, calculated_pre_state_root);
        assert_eq!(post_state_root, calculated_post_state_root);
    }
}
