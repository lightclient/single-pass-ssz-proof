#![cfg_attr(not(feature = "std"), no_std)]

mod utils;
use utils::hash;

const TREE_SIZE: usize = 4;
const NUM_HASHES: usize = TREE_SIZE + 1;

// AccountBalance => FixedVector[u256, 16];
//
// Before updating, all account's have a balance of 0. Therefore, the accounts' root before updating
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

fn main() {
    // Bytes representation of `zh(4)` (e.g. "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c").
    let pre_state_root: [u8; 32] = [
        83, 109, 152, 131, 127, 45, 209, 101, 165, 93, 94, 234, 233, 20, 133, 149, 68, 114, 213,
        111, 36, 109, 242, 86, 191, 60, 174, 25, 53, 42, 18, 60,
    ];

    // The account which the proof is for and whose balance will be updated
    let account: usize = 0;

    // Merkle proof for an account with a balance of 0. Since all account have a balance of 0, this
    // initial proof can be used interchangably amongst them.
    let input: [&[u8; 32]; 5] = [
        // Account leaf
        &[0u8; 32],
        // Sibling account
        &[0u8; 32],
        // zh(1) sister
        &[
            245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67, 0, 61,
            35, 32, 217, 240, 232, 234, 152, 49, 169, 39, 89, 251, 75,
        ],
        // zh(2) sister
        &[
            219, 86, 17, 78, 0, 253, 212, 193, 248, 92, 137, 43, 243, 90, 201, 168, 146, 137, 170,
            236, 177, 235, 208, 169, 108, 222, 96, 106, 116, 139, 93, 113,
        ],
        // zh(3) sister
        &[
            199, 128, 9, 253, 240, 127, 197, 106, 17, 241, 34, 55, 6, 88, 163, 83, 170, 165, 66,
            237, 99, 228, 76, 75, 193, 95, 244, 205, 16, 90, 179, 60,
        ],
    ];

    // The new balance will `1u256`.
    let new_balance: [u8; 32] = {
        let mut tmp = [0u8; 32];
        tmp[0] = 1;
        tmp
    };

    // Buffer to hold the calculated post-state root
    let mut post_state_root = [0u8; 32];

    // Calculate the post-state root and verify the pre-state root in one pass.
    single_pass(
        account + 16,
        &input,
        &pre_state_root,
        &mut post_state_root,
        &new_balance,
    );

    #[cfg(feature = "std")]
    println!(
        "pre-state root:  {}\npost-state root: {}",
        hex::encode(&pre_state_root),
        hex::encode(&post_state_root)
    );
}

// A few caveats at the moment:
//
// i) Only supports a single branch.
fn single_pass(
    mut index: usize,
    chunks: &[&[u8; 32]; NUM_HASHES],
    pre_state_root: &[u8; 32],
    post_state_root_buf: &mut [u8; 32],
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
                "last calculated: {}\nh({}, {})\npre-state:  {:?} | {:?}\npost-state: {:?} | {:?}\n",
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

    // Verify that the calculated pre-state root is equal to the expected pre-state root.
    assert_eq!(pre_state_root, &pre_buf[0..32]);

    // Return the calculated post-state root.
    post_state_root_buf.copy_from_slice(&post_buf[0..32]);
}
