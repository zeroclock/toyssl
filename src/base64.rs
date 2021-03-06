const BASE64: [char; 64] = ['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','0','1','2','3','4','5','6','7','8','9','+','/'];
const BASE64_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(input: &str) -> String {
    let input_bytes = input.as_bytes();
    let mut bits = String::new();
    input_bytes.iter().for_each(|byte| {
        bits = format!("{}{:08b}", bits, byte);
    });
    println!("{}", bits);
    let last_idx = bits.len();
    let mut idx: usize = 0;
    let mut result = String::new();
    while last_idx > idx {
        let end = idx.saturating_add(6);
        if last_idx < end {
            let pos = usize::from_str_radix(
                &format!("{:0<6}", &bits[idx..last_idx]),
                2
            ).unwrap();
            result = format!("{}{}", result, BASE64[pos]);
            break;
        } else {
            let pos = usize::from_str_radix(
                &format!("{}", &bits[idx..end]),
                2
            ).unwrap();
            result = format!("{}{}", result, BASE64[pos]);
        }
        idx = end;
    }
    if result.len() % 4 > 0 {
        for _ in 0..(4 - (result.len() % 4)) {
            result = format!("{}=", result);
        }
    }
    // println!("{}", result);
    result
}

pub fn base64_decode(input: &str) -> String {
    let mut result = String::new();
    let mut bits = String::new();
    for c in input.chars() {
        let base64_pos = BASE64_STR.find(c);
        if let Some(pos) = base64_pos {
            bits = format!("{}{:0>6b}", bits, pos);
        } else {
            bits = format!("{}000000", bits);
        }
    }
    let mut idx: usize = 0;
    println!("len: {}", bits.len());
    while idx < bits.len() {
        println!("idx: {}", idx);
        let utf8: &[u8] = &[
            u8::from_str_radix(
                &bits[idx..idx.saturating_add(8)],
                2,
            ).unwrap()
        ];
        if utf8[0] != 0 {
            let s = std::str::from_utf8(&utf8).unwrap();
            result = format!("{}{}", result, s);
        }
        idx = idx.saturating_add(8);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_base64_encode_input_1() {
        let actual = base64_encode("thisistestinput");
        let expected = "dGhpc2lzdGVzdGlucHV0".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_base64_encode_input_2() {
        let actual = base64_encode("username:password");
        let expected = "dXNlcm5hbWU6cGFzc3dvcmQ=".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_base64_encode_input_3() {
        let actual = base64_encode("!base64_encoding_is_awesome?");
        let expected = "IWJhc2U2NF9lbmNvZGluZ19pc19hd2Vzb21lPw==".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_base64_decode_input_1() {
        let actual = base64_decode("dGhpc2lzdGVzdGlucHV0");
        let expected = "thisistestinput".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_base64_decode_input_2() {
        let actual = base64_decode("dXNlcm5hbWU6cGFzc3dvcmQ=");
        let expected = "username:password".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_can_base64_decode_input_3() {
        let actual = base64_decode("IWJhc2U2NF9lbmNvZGluZ19pc19hd2Vzb21lPw==");
        let expected = "!base64_encoding_is_awesome?".to_string();
        assert_eq!(expected, actual);
    }
}
