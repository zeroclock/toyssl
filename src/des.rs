use anyhow::{
    Result,
    anyhow,
};

const DES_BLOCK_SIZE: u8 = 8; // 64 bits, defined in the standard
const DES_KEY_SIZE: u8 = 8; // 56 bits used, but must supply 64 (8 are ignored)

const INITIAL_PERM_TABLE: [u8; 64] = [
    58, 50, 42, 34, 26, 18, 10, 2,
    60, 52, 44, 36, 28, 20, 12, 4,
    62, 54, 46, 38, 30, 22, 14, 6,
    64, 56, 48, 40, 32, 24, 16, 8,
    57, 49, 41, 33, 25, 17, 9, 1,
    59, 51, 43, 35, 27, 19, 11, 3,
    61, 53, 45, 37, 29, 21, 13, 5,
    63, 55, 47, 39, 31, 23, 15, 7,
];

const FINAL_PERM_TABLE: [u8; 64] = [
    40, 8, 48, 16, 56, 24, 64, 32,
    39, 7, 47, 15, 55, 23, 63, 31,
    38, 6, 46, 14, 54, 22, 62, 30,
    37, 5, 45, 13, 53, 21, 61, 29,
    36, 4, 44, 12, 52, 20, 60, 28,
    35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26,
    33, 1, 41, 9, 49, 17, 57, 25,
];

const KEY_PERM_TABLE_1: [u8; 56] = [
    57, 49, 41, 33, 25, 17, 9, 1,
    58, 50, 42, 34, 26, 18, 10, 2,
    59, 51, 43, 35, 27, 19, 11, 3,
    60, 52, 44, 36,
    63, 55, 47, 39, 31, 23, 15, 7,
    62, 54, 46, 38, 30, 22, 14, 6,
    61, 53, 45, 37, 29, 21, 13, 5,
    28, 20, 12, 4,
];

const KEY_PERM_TABLE_2: [u8; 48] = [
    14, 17, 11, 24, 1, 5,
    3, 28, 15, 6, 21, 10,
    23, 19, 12, 4, 26, 8,
    16, 7, 27, 20, 13, 2,
    41, 52, 31, 37, 47, 55,
    30, 40, 51, 45, 33, 48,
    44, 49, 39, 56, 34, 53,
    46, 42, 50, 36, 29, 32,
];

const INPUT_EXPANSION_TABLE: [u8; 48] = [
    32, 1, 2, 3, 4, 5,
    4, 5, 6, 7, 8, 9,
    8, 9, 10, 11, 12, 13,
    12, 13, 14, 15, 16, 17,
    16, 17, 18, 19, 20, 21,
    20, 21, 22, 23, 24, 25,
    24, 25, 26, 27, 28, 29,
    28, 29, 30, 31, 32, 1,
];

/// Sample s-box table
/// The values ​​in the table can be anything as long as they are completely random
/// Notice that s-box table values must not be come from a linear function! 
const SBOX_TABLE: [[u8; 64]; 8] = [
    [
        14, 0, 4, 15, 13, 7, 1, 4, 2, 14, 15, 2, 11, 13, 8, 1,
        3, 10, 10, 6, 6, 12, 12, 11, 5, 9, 9, 5, 0, 3, 7, 8,
        4, 15, 1, 12, 14, 8, 8, 2, 13, 4, 6, 9, 2, 1, 11, 7,
        15, 5, 12, 11, 9, 3, 7, 14, 3, 10, 10, 0, 5, 6, 0, 13,
    ],
    [
        15, 3, 1, 13, 8, 4, 14, 7, 6, 15, 11, 2, 3, 8, 4, 14,
        9, 12, 7, 0, 2, 1, 13, 10, 12, 6, 0, 9, 5, 11, 10, 5,
        0, 13, 14, 8, 7, 10, 11, 1, 10, 3, 4, 15, 13, 4, 1, 2,
        5, 11, 8, 6, 12, 7, 6, 12, 9, 0, 3, 5, 2, 14, 15, 9,
    ],
    [
        10, 13, 0, 7, 9, 0, 14, 9, 6, 3, 3, 4, 15, 6, 5, 10,
        1, 2, 13, 8, 12, 5, 7, 14, 11, 12, 4, 11, 2, 15, 8, 1,
        13, 1, 6, 10, 4, 13, 9, 0, 8, 6, 15, 9, 3, 8, 0, 7,
        11, 4, 1, 15, 2, 14, 12, 3, 5, 11, 10, 5, 14, 2, 7, 12,
    ],
    [
        7, 13, 13, 8, 14, 11, 3, 5, 0, 6, 6, 15, 9, 0, 10, 3,
        1, 4, 2, 7, 8, 2, 5, 12, 11, 1, 12, 10, 4, 14, 15, 9,
        10, 3, 6, 15, 9, 0, 0, 6, 12, 10, 11, 1, 7, 13, 13, 8,
        15, 9, 1, 4, 3, 5, 14, 11, 5, 12, 2, 7, 8, 2, 4, 14,
    ],
    [
        2, 14, 12, 11, 4, 2, 1, 12, 7, 4, 10, 7, 11, 13, 6, 1,
        8, 5, 5, 0, 3, 15, 15, 10, 13, 3, 0, 9, 14, 8, 9, 6,
        4, 11, 2, 8, 1, 12, 11, 7, 10, 1, 13, 14, 7, 2, 8, 13,
        15, 6, 9, 15, 12, 0, 5, 9, 6, 10, 3, 4, 0, 5, 14, 3,
    ],
    [
        12, 10, 1, 15, 10, 4, 15, 2, 9, 7, 2, 12, 6, 9, 8, 5,
        0, 6, 13, 1, 3, 13, 4, 14, 14, 0, 7, 11, 5, 3, 11, 8,
        9, 4, 14, 3, 15, 2, 5, 12, 2, 9, 8, 5, 12, 15, 3, 10,
        7, 11, 0, 14, 4, 1, 10, 7, 1, 6, 13, 0, 11, 8, 6, 13,
    ],
    [
        4, 13, 11, 0, 2, 11, 14, 7, 15, 4, 0, 9, 8, 1, 13, 10,
        3, 14, 12, 3, 9, 5, 7, 12, 5, 2, 10, 15, 6, 8, 1, 6,
        1, 6, 4, 11, 11, 13, 13, 8, 12, 1, 3, 4, 7, 10, 14, 7,
        10, 9, 15, 5, 6, 0, 8, 15, 0, 14, 5, 2, 9, 3, 2, 12,
    ],
    [
        13, 1, 2, 15, 8, 13, 4, 8, 6, 10, 15, 3, 11, 7, 1, 4,
        10, 12, 9, 5, 3, 6, 14, 11, 5, 0, 0, 14, 12, 9, 7, 2,
        7, 2, 11, 1, 4, 14, 1, 7, 9, 4, 12, 10, 14, 8, 2, 13,
        0, 15, 6, 12, 10, 9, 13, 0, 15, 3, 3, 5, 5, 6, 8, 11,
    ]
];

const FINAL_INPUT_PERM_TABLE: [u8; 32] = [
    16, 7, 20, 21,
    29, 12, 28, 17,
    1, 15, 23, 26,
    5, 18, 31, 10,
    2, 8, 24, 14,
    32, 27, 3, 9,
    19, 13, 30, 6,
    22, 11, 4, 25,
];

pub enum OpType {
    ENCRYPT,
    DECRYPT,
}

pub fn des_block_operate(plain_data: &mut Vec<u8>, key_data: &Vec<u8>, op_type: OpType) -> Result<Vec<u8>> {
    if plain_data.len() != DES_BLOCK_SIZE as usize {
        return Err(anyhow!("plain_data length is incorrect. expected: {}, actual: {}", DES_BLOCK_SIZE, plain_data.len()));
    }
    if key_data.len() != DES_KEY_SIZE as usize {
        return Err(anyhow!("key length is incorrect. expected: {}, actual: {}", DES_KEY_SIZE, key_data.len()));
    }
    // Initial permutation
    let mut input = permute(plain_data, INITIAL_PERM_TABLE.to_vec().as_ref())?;

    // Key schedule computation
    let mut key = permute(key_data, KEY_PERM_TABLE_1.to_vec().as_ref())?;
    for round in 0..16 {
        // Fiestel function on the first half of the block in input

        // "Expansion". This permutation only look at the first
        // four bytes (32 bits of input); 16 of these are repeated
        // in "INPUT_EXPANSION_TABLE".
        let mut expanded_input = permute(input[4..].to_vec().as_ref(), INPUT_EXPANSION_TABLE.to_vec().as_ref())?;
        // "Key mixing"
        // rotate both halves of the initial key
        if let OpType::ENCRYPT = op_type {
            key_rotate_left(&mut key)?;
            if !(round <= 1 || round == 8 || round == 15) {
                // Rotate twice except in rounds 1, 2, 9 & 16
                key_rotate_left(&mut key)?;
            }
        }
        let sub_key = permute(&key, KEY_PERM_TABLE_2.to_vec().as_ref())?;
        if let OpType::DECRYPT = op_type {
            key_rotate_right(&mut key)?;
            if !(round >= 14 || round == 7 || round == 0) {
                // Rotate twice except in rounds 1, 2, 9 & 16
                key_rotate_right(&mut key)?;
            }
        }
        expanded_input = xor(&expanded_input, &sub_key)?;

        // Substitution
        let mut substitution_block: Vec<u8> = vec![0, 0, 0, 0];
        // Processing expanded_input (8 bytes) by 6 bits each
        substitution_block[0] = SBOX_TABLE[0][((expanded_input[0] & 0xFC) >> 2) as usize] << 4;
        substitution_block[0] |= SBOX_TABLE[1][((expanded_input[0] & 0x03) << 4 | (expanded_input[1] & 0xF0) >> 4) as usize];
        substitution_block[1] = SBOX_TABLE[2][((expanded_input[1] & 0x0F) << 2 | (expanded_input[2] & 0xC0) >> 6) as usize] << 4;
        substitution_block[1] |= SBOX_TABLE[3][(expanded_input[2] & 0x3F) as usize];
        substitution_block[2] = SBOX_TABLE[4][((expanded_input[3] & 0xFC) >> 2) as usize] << 4;
        substitution_block[2] |= SBOX_TABLE[5][((expanded_input[3] & 0x03) << 4 | (expanded_input[4] & 0xF0) >> 4) as usize];
        substitution_block[3] = SBOX_TABLE[6][((expanded_input[4] & 0x0F) << 2 | (expanded_input[5] & 0xC0) >> 6) as usize] << 4;
        substitution_block[3] |= SBOX_TABLE[7][(expanded_input[5] & 0x3F) as usize];

        // Permutation
        let shrinked_input = permute(&substitution_block, FINAL_INPUT_PERM_TABLE.to_vec().as_ref())?;

        // Recombination. XOR the shrinked_input with left half and then switch sides.
        let mut left_side = input[..(DES_BLOCK_SIZE / 2) as usize].to_vec();
        let mut right_side = input[((DES_BLOCK_SIZE / 2) as usize)..].to_vec();
        left_side = xor(&left_side, &shrinked_input)?;
        // swap each side
        input = vec![];
        input.append(&mut right_side);
        input.append(&mut left_side);
    }

    // Swap one last time
    let mut left_side = input[..(DES_BLOCK_SIZE / 2) as usize].to_vec();
    let mut right_side = input[((DES_BLOCK_SIZE / 2) as usize)..].to_vec();
    input = vec![];
    input.append(&mut right_side);
    input.append(&mut left_side);

    // Final permutation (undo initial permutation)
    let result = permute(&input, FINAL_PERM_TABLE.to_vec().as_ref())?;
    
    Ok(result)
}

fn permute(src: &Vec<u8>, permute_table: &Vec<u8>) -> Result<Vec<u8>> {
    let mut max = 0;
    permute_table.iter().for_each(|v| {
        if &max < v {
            max = *v;
        }
    });
    if max > (src.len() * 8) as u8 {
        return Err(anyhow!("length of src length was expected to be {} at least. passed: {}", max, src.len()));
    }
    let mut result: Vec<u8> = vec![0; permute_table.len() / 8];
    for i in 0..permute_table.len() {
        if get_bit(src, (permute_table[i] - 1) as usize) {
            set_bit(&mut result, i);
        } else {
            clear_bit(&mut result, i);
        }
    }
    Ok(result)
}

fn get_bit(bytes: &[u8], bit: usize) -> bool {
    bytes[bit / 8 as usize] & 0x80 >> (bit % 8) > 0
}

fn set_bit(bytes: &mut Vec<u8>, bit: usize) {
    bytes[bit / 8 as usize] |= 0x80 >> (bit % 8);
}

fn clear_bit(bytes: &mut Vec<u8>, bit: usize) {
    bytes[bit / 8 as usize] &= !(0x80 >> (bit % 8));
}

fn xor(bytes_1: &Vec<u8>, bytes_2: &Vec<u8>) -> Result<Vec<u8>> {
    if bytes_1.len() != bytes_2.len() {
        return Err(anyhow!("bytes_1 and bytes_2 length are wrong. bytes_1.len(): {}, bytes_2.len(): {}", bytes_1.len(), bytes_2.len()));
    }
    let mut result: Vec<u8> = vec![];
    for i in 0..bytes_1.len() {
        result.push(bytes_1[i] ^ bytes_2[i]);
    }
    Ok(result)
}

/// rotate key (left)
/// notice that the key is split into two 28-bit halves and each of which has to be rotated independently
/// For example:                        \/<-----split point
/// in  : 01100111 01000111 00011100 00101001 00010110 10111101 01011000
/// out : 11001110 10001110 00111000 01010010 00101101 01111010 10110001
fn key_rotate_left(bytes: &mut Vec<u8>) -> Result<()> {
    if bytes.len() != 7 {
        return Err(anyhow!("Key length is incorrect. expected: {}, actual: {}", 7, bytes.len()));
    }
    // store overflowed bit (left half)
    let carry_left: u8 = (bytes[0] & 0x80) >> 3;
    // left shift
    bytes[0] = (bytes[0] << 1) | (bytes[1] & 0x80) >> 7;
    bytes[1] = (bytes[1] << 1) | (bytes[2] & 0x80) >> 7;
    bytes[2] = (bytes[2] << 1) | (bytes[3] & 0x80) >> 7;
    // special handling for byte 3 (it has split point)
    // store overflowed bit (right half)
    let carry_right: u8 = (bytes[3] & 0x08) >> 3;
    bytes[3] = (((bytes[3] << 1) | ((bytes[4] & 0x80) >> 7)) & !0x10) | carry_left;
    // left shift
    bytes[4] = (bytes[4] << 1) | (bytes[5] & 0x80) >> 7;
    bytes[5] = (bytes[5] << 1) | (bytes[6] & 0x80) >> 7;
    bytes[6] = (bytes[6] << 1) | carry_right;
    Ok(())
}

/// rotate key (right)
/// notice that the key is split into two 28-bit halves and each of which has to be rotated independently
/// For example:                        \/<-----split point
/// in  : 01100111 01000111 00011100 00101001 00010110 10111101 01011000
/// out : 00110011 10100011 10001110 00010100 10001011 01011110 10101100
fn key_rotate_right(bytes: &mut Vec<u8>) -> Result<()> {
    if bytes.len() != 7 {
        return Err(anyhow!("Key length is incorrect. expected: {}, actual: {}", 7, bytes.len()));
    }

    let carry_right: u8 = (bytes[6] & 0x01) << 3;

    bytes[6] = (bytes[6] >> 1) | (bytes[5] & 0x01) << 7;
    bytes[5] = (bytes[5] >> 1) | (bytes[4] & 0x01) << 7;
    bytes[4] = (bytes[4] >> 1) | (bytes[3] & 0x01) << 7;

    let carry_left: u8 = (bytes[3] & 0x10) << 3;
    bytes[3] = (((bytes[3] >> 1) | ((bytes[2] & 0x01) << 7)) & !0x08) | carry_right;

    bytes[2] = (bytes[2] >> 1) | (bytes[1] & 0x01) << 7;
    bytes[1] = (bytes[1] >> 1) | (bytes[0] & 0x01) << 7;
    bytes[0] = (bytes[0] >> 1) | carry_left;
    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_can_permute_same_length() {
        // in  : 10101010 10101010 10101010 10101010 10101010 10101010 10101010 10101010 (170, 170, ...)
        // out : 00000000 00000000 00000000 00000000 11111111 11111111 11111111 11111111 (0, ..., 255, ...)
        let perm_table: Vec<u8> = vec![
            58, 50, 42, 34, 26, 18, 10, 2,
            60, 52, 44, 36, 28, 20, 12, 4,
            62, 54, 46, 38, 30, 22, 14, 6,
            64, 56, 48, 40, 32, 24, 16, 8,
            57, 49, 41, 33, 25, 17, 9, 1,
            59, 51, 43, 35, 27, 19, 11, 3,
            61, 53, 45, 37, 29, 21, 13, 5,
            63, 55, 47, 39, 31, 23, 15, 7,
        ];
        let input_data: Vec<u8> = vec![170, 170, 170, 170, 170, 170, 170, 170];
        let expected_data: Vec<u8> = vec![0, 0, 0, 0, 255, 255, 255, 255];
        let result = permute(&input_data, &perm_table).unwrap();
        assert_eq!(expected_data, result);
        let expected_data: Vec<u8> = vec![240, 240, 240, 240, 240, 240, 240, 240];
        let result2 = permute(&result, &perm_table).unwrap();
        assert_eq!(expected_data, result2);
    }

    #[test]
    fn test_can_permute_src_length_larger() {
        // in  : 10101010 10101010 10101010 10101010 10101010 10101010 10101010 10101010 (170, 170, ...)
        // out : 00000000 00000000 11111111 11111111
        let perm_table: Vec<u8> = vec![
            58, 50, 42, 34, 26, 18, 10, 2,
            60, 52, 44, 36, 28, 20, 12, 4,
            57, 49, 41, 33, 25, 17, 9, 1,
            59, 51, 43, 35, 27, 19, 11, 3,
        ];
        let input_data: Vec<u8> = vec![170, 170, 170, 170, 170, 170, 170, 170];
        let expected_data: Vec<u8> = vec![0, 0, 255, 255];
        let result = permute(&input_data, &perm_table).unwrap();
        assert_eq!(expected_data, result);
    }

    #[test]
    fn test_can_permute_src_length_shorter() {
        // in  : 10101010 10101010
        // out : 00000000 11111111
        let perm_table: Vec<u8> = vec![
            16, 14, 12, 10, 8, 6, 4, 2,
            15, 13, 11, 9, 7, 5, 3, 1,
        ];
        let input_data: Vec<u8> = vec![170, 170];
        let expected_data: Vec<u8> = vec![0, 255];
        let result = permute(&input_data, &perm_table).unwrap();
        assert_eq!(expected_data, result);
    }

    #[test]
    fn test_can_xor() {
        // in  : 01100111 10010100 (103, 148)
        //       xor
        //       10001011 10111010 (139, 186)
        // out : 11101100 00101110 (236, 46)
        let input_1: Vec<u8> = vec![103, 148];
        let input_2: Vec<u8> = vec![139, 186];
        let expected: Vec<u8> = vec![236, 46];
        let actual: Vec<u8> = xor(&input_1, &input_2).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_rotate_key_left() {
        // in  : 01100111 01000111 00011100 00101001 00010110 10111101 01011000 (103, 71, 28, 41, 22, 189, 88)
        // out : 11001110 10001110 00111000 01000010 00101101 01111010 10110001 (206, 142, 56, 66, 45, 122, 177)
        let mut input: Vec<u8> = vec![103, 71, 28, 41, 22, 189, 88];
        let expected: Vec<u8> = vec![206, 142, 56, 66, 45, 122, 177];
        key_rotate_left(&mut input).unwrap();
        assert_eq!(expected, input);
    }

    #[test]
    fn test_can_rotate_key_right() {
        // in  : 01100111 01000111 00011100 00111001 00010110 10111101 01011000 (103, 71, 28, 57, 22, 189, 88)
        // out : 10110011 10100011 10001110 00010100 10001011 01011110 10101100 (179, 163, 142, 20, 139, 94, 172)
        let mut input: Vec<u8> = vec![103, 71, 28, 57, 22, 189, 88];
        let expected: Vec<u8> = vec![179, 163, 142, 20, 139, 94, 172];
        key_rotate_right(&mut input).unwrap();
        assert_eq!(expected, input);
    }

    #[test]
    fn test_can_des_operate() {
        let plain_text = "abcdefgh";
        let key_text = "keyisokk";
        let wrong_key_text = "keyisokK";
        let mut plain_data = plain_text.as_bytes().to_vec();
        let key_data = key_text.as_bytes().to_vec();
        let wrong_key_data = wrong_key_text.as_bytes().to_vec();
        let mut result_encrypted = des_block_operate(&mut plain_data, &key_data, OpType::ENCRYPT).unwrap();
        let result_decrypted = des_block_operate(&mut result_encrypted, &key_data, OpType::DECRYPT).unwrap();
        let result_wrong_decrypted = des_block_operate(&mut result_encrypted, &wrong_key_data, OpType::DECRYPT).unwrap();
        assert_eq!(plain_data, result_decrypted);
        assert_ne!(plain_data, result_wrong_decrypted);
    }
}

