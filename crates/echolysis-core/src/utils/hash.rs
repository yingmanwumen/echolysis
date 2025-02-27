/// Merges two 64-bit hash values into a single 64-bit hash value.
///
/// This function takes two 64-bit hash values, `lhs` and `rhs`, and combines them
/// using a specific algorithm to produce a single 64-bit hash value. The algorithm
/// involves using a seed value, multiplying and XORing the input values, and applying
/// a final adjustment to ensure the result is not equal to `u64::MAX`.
///
/// # Arguments
///
/// * `lhs` - The first 64-bit hash value.
/// * `rhs` - The second 64-bit hash value.
///
/// # Returns
///
/// A 64-bit hash value that represents the combined hash of `lhs` and `rhs`.
pub fn merge_structure_hash(lhs: u64, rhs: u64) -> u64 {
    // Initialize with a fixed seed value - a 64-bit hexadecimal constant
    let seed: u64 = 0x0123456789abcdef;

    // First mixing step:
    // 1. Add 0x01 to the first hash to ensure non-zero
    // 2. Multiply seed by prime number 1000003 using wrapping multiplication to handle overflow
    // 3. XOR with the modified first hash
    let mut value = seed.wrapping_mul(1000003) ^ (lhs + 0x01);

    // Second mixing step:
    // 1. Add 0x02 to the second hash (different offset than first hash)
    // 2. Multiply previous value by same prime using wrapping multiplication
    // 3. XOR with the modified second hash
    value = value.wrapping_mul(1000003) ^ (rhs + 0x02);

    // Final mixing step - XOR with 2 to further scramble bits
    value ^= 2;

    // Ensure the final hash is never equal to u64::MAX
    // This is important for some hash table implementations
    // that reserve MAX as a special value
    if value == u64::MAX {
        u64::MAX - 1
    } else {
        value
    }
}
