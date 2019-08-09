use crate::utils::hash;
use crate::CHUNK_SIZE;

// A few caveats at the moment:
//
// i) Only supports a single branch.
pub fn update(
    mut index: u64,
    chunks: &[u8],
    calculated_pre_state_root: &mut [u8; CHUNK_SIZE],
    calculated_post_state_root: &mut [u8; CHUNK_SIZE],
    new_balance: &[u8; CHUNK_SIZE],
) {
    // Initialize buffers to hold the different calculations. The first 32 bytes are the left
    // chunk and the second 32 bytes is the right chunk.
    let mut pre_buf = [0u8; 2 * CHUNK_SIZE];
    let mut post_buf = [0u8; 2 * CHUNK_SIZE];

    // Copy the starting bytes for both buffers. They will be more into the correct slot later
    // depending on the parity of the leaf.
    pre_buf[0..CHUNK_SIZE].copy_from_slice(&chunks[0..CHUNK_SIZE]);
    post_buf[0..CHUNK_SIZE].copy_from_slice(new_balance);

    // Begin calculating the root -- skip the first chunk since it has already been loaded into the
    // buffer.
    let mut i = CHUNK_SIZE;
    while i < chunks.len() {
        // The leaf's parity determines which slot the chunk data should go.
        // Even => Left node  => 0..32
        // Odd  => Right node => 32..64
        let parity = (index % 2) as usize;

        // If the last calculated hash was for an odd node, it should be in the second 32 bytes of
        // the hash buffer.
        if parity == 1 {
            let mut tmp = [0u8; CHUNK_SIZE];
            tmp.copy_from_slice(&pre_buf[0..CHUNK_SIZE]);
            pre_buf[CHUNK_SIZE..2 * CHUNK_SIZE].copy_from_slice(&tmp);

            tmp.copy_from_slice(&post_buf[0..CHUNK_SIZE]);
            post_buf[CHUNK_SIZE..2 * CHUNK_SIZE].copy_from_slice(&tmp);
        }

        // Copy the sibling chunk into the buffer to be hashed. Xor the leaf's parity to get the
        // sibling's parity.
        let begin = CHUNK_SIZE * (parity ^ 1);
        let end = CHUNK_SIZE * (parity ^ 1) + CHUNK_SIZE;
        pre_buf[begin..end].copy_from_slice(&chunks[i..(i + CHUNK_SIZE)]);
        post_buf[begin..end].copy_from_slice(&chunks[i..(i + CHUNK_SIZE)]);

        #[cfg(feature = "std")]
        {
            let left = index & (-2isize as u64);
            let right = left + 1;
            println!(
                "last calculated: {}\nh({}, {})\npre-state:  {} | {}\npost-state: {} | {}\n",
                index,
                left,
                right,
                hex::encode(&pre_buf[0..CHUNK_SIZE]),
                hex::encode(&pre_buf[CHUNK_SIZE..2 * CHUNK_SIZE]),
                hex::encode(&post_buf[0..CHUNK_SIZE]),
                hex::encode(&post_buf[CHUNK_SIZE..2 * CHUNK_SIZE]),
            );
        }

        // The hash function will hash all 64 bytes & replace the first 32 bytes with the new hash.
        hash(&mut pre_buf);
        hash(&mut post_buf);

        i += 32;
        index = index / 2;
    }

    // Return the calculated post-state root.
    calculated_pre_state_root.copy_from_slice(&pre_buf[0..CHUNK_SIZE]);
    calculated_post_state_root.copy_from_slice(&post_buf[0..CHUNK_SIZE]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::generate_branch;

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
        let account: u64 = 0;

        let mut input = [0u8; 5 * CHUNK_SIZE];

        // Generate a merkle proof for an account with a balance of 0. Since all account have a
        // balance of 0, this initial proof can be used interchangably amongst them.
        generate_branch(4, &mut input);

        // The new balance will `1u256`.
        let new_balance: [u8; CHUNK_SIZE] = {
            let mut tmp = [0u8; CHUNK_SIZE];
            tmp[0] = 1;
            tmp
        };

        // Buffer to hold the calculated post-state root
        let mut calculated_pre_state_root = [0u8; CHUNK_SIZE];
        let mut calculated_post_state_root = [0u8; CHUNK_SIZE];

        // Calculate the post-state root and verify the pre-state root in one pass.
        update(
            account + (1 << 4),
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
                .unwrap()[0..CHUNK_SIZE];
        let post_state_root =
            &hex::decode("0eefb94faea1cefdd28c895a51ba5822bcac513cd59413ece941e21d78bc83c4")
                .unwrap()[0..CHUNK_SIZE];

        assert_eq!(pre_state_root, calculated_pre_state_root);
        assert_eq!(post_state_root, calculated_post_state_root);
    }

    #[test]
    fn sanity_check_bigger() {
        // For this example, we'll assume we're operating on an object with the following
        // structure:
        //
        // AccountBalance => FixedVector[u256, 2**40];
        //
        // All account's start with a balance of 0. Therefore, the accounts' root before updating
        // is `zh(40)`.

        // The account which the proof is for and whose balance will be updated
        let account: u64 = 99999;

        // Intialize buffer to hold branch with 41 nodes.
        let mut input = [0u8; 41 * CHUNK_SIZE];

        // Generate a merkle proof for an account with a balance of 0. Since all account have a
        // balance of 0, this initial proof can be used interchangably amongst them.
        generate_branch(40, &mut input);
        #[cfg(feature = "std")]
        println!("input data: {}", hex::encode(&input[0..1312]));

        // The new balance will `1u256`.
        let new_balance: [u8; CHUNK_SIZE] = {
            let mut tmp = [0u8; CHUNK_SIZE];
            tmp[0] = 1;
            tmp
        };

        // Buffer to hold the calculated post-state root
        let mut calculated_pre_state_root = [0u8; CHUNK_SIZE];
        let mut calculated_post_state_root = [0u8; CHUNK_SIZE];

        // Calculate the post-state root and verify the pre-state root in one pass.
        update(
            // 1 << 40 == first leaf index
            account + (1 << 40),
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
            &hex::decode("6bfe8d2bcc4237b74a5047058ef455339ecd7360cb63bfbb8ee5448e6430ba04")
                .unwrap()[0..CHUNK_SIZE];
        let post_state_root =
            &hex::decode("d9d47ae1800a35de6007a8541eded6e0ede2826c5da208709ed45eca6a1c16c4")
                .unwrap()[0..CHUNK_SIZE];

        assert_eq!(pre_state_root, calculated_pre_state_root);
        assert_eq!(post_state_root, calculated_post_state_root);
    }
}
