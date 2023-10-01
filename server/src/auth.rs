use hex_literal::hex;
use pbkdf2::pbkdf2;
use pbkdf2::hmac::Hmac;
use sha2::Sha256;

fn encrypt(input_str: &[u8]) -> [u8; 20] {
    let mut dest = [0u8; 20];
    pbkdf2::<Hmac<Sha256>>(input_str, b"salt", 600_000, &mut dest)
        .expect("HMAC can be initialized with any key length");
    return dest;
}

mod test {
    use super::*;

    #[test]
    fn test_encrypt() {
        let input_str = b"password";
        let out = encrypt(input_str);
        assert_eq!(out, hex!("669cfe52482116fda1aa2cbe409b2f56c8e45637"));
    }
}