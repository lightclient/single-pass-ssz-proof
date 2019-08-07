mod utils;
use utils::hash;

const TREE_SIZE: usize = 4;
const NUM_HASHES: usize = TREE_SIZE + 1;

// AccountBalance => FixedVector[u256, 16];
//
// Before updating, all account's have a balance of 0. Therefore, the accounts' root before updating
// is `zh(4) == "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c"`.
//

//
//                 +-------- 0 --------+                <= zh(4)
//                /                     \
//           +-- 1 --+               +-- 2 --+          <= zh(3)
//          /         \             /         \
//         3           4           5           6        <= zh(2)
//       /   \       /   \       /   \       /  \
//      7     8     9    10     11   12     13   14     <= zh(1)
//     / \   / \   / \   / \   / \   / \   / \   / \
//    15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30   <= zh(0)

fn main() {
    // Bytes representation of "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c".
    let pre_state_root: [u8; 32] = [
        83, 109, 152, 131, 127, 45, 209, 101, 165, 93, 94, 234, 233, 20, 133, 149, 68, 114, 213,
        111, 36, 109, 242, 86, 191, 60, 174, 25, 53, 42, 18, 60,
    ];

    // Merkle proof for the account at index `0`, which includes the chunks at these indexes:
    // [15, 16, 8, 4, 2]. The balance of accounts `0` and `1` is `0`.
    let input: [&[u8; 32]; 5] = [
        &[0u8; 32],
        &[0u8; 32],
        &[
            245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67, 0, 61,
            35, 32, 217, 240, 232, 234, 152, 49, 169, 39, 89, 251, 75,
        ],
        &[
            219, 86, 17, 78, 0, 253, 212, 193, 248, 92, 137, 43, 243, 90, 201, 168, 146, 137, 170,
            236, 177, 235, 208, 169, 108, 222, 96, 106, 116, 139, 93, 113,
        ],
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
    single_pass(&input, &pre_state_root, &mut post_state_root, &new_balance);

    println!("pre-state root:  {}", hex::encode(&pre_state_root));
    println!("post-state root: {}", hex::encode(&post_state_root));
}

// A few caveats at the moment:
//
// i)   This only works if the intial balance of the account is `0`.
// ii)  The logic is hard-coded for the left most account (e.g. index `0`).
// ii)  Only supports a single branch.
fn single_pass(
    chunks: &[&[u8; 32]; NUM_HASHES],
    pre_state_root: &[u8; 32],
    post_state_root_buf: &mut [u8; 32],
    new_balance: &[u8; 32],
) {
    // Initialize buffers to hold the different calculations. The first 32 bytes are the right
    // chunk and the second 32 bytes is the left chunk.
    let mut pre_buf = [0u8; 64];
    let mut post_buf = [0u8; 64];

    // Start by copying the new balance into the left chunk's slot. Because we're starting in the
    // left slot, this will only work for updating the balance of odd numbered accounts.
    post_buf[0..32].copy_from_slice(new_balance);

    // Begin calculating the root -- skip the first chunk since it is either `0` or `new_balance`.
    for chunk in chunks.iter().skip(1) {
        // Copy the sibling chunk into the buffer to be hashed.
        pre_buf[32..64].copy_from_slice(*chunk);
        post_buf[32..64].copy_from_slice(*chunk);

        // The hash function will hash all 64 bytes & replace the first 32 bytes with the new hash.
        hash(&mut pre_buf);
        hash(&mut post_buf);
    }

    // Verify that the calculated pre-state root is equal to the expected pre-state root.
    assert_eq!(pre_state_root, &pre_buf[0..32]);

    // Return the calculated post-state root.
    post_state_root_buf.copy_from_slice(&post_buf[0..32]);
}
