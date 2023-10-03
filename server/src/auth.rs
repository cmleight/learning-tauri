use hex_literal::hex;
use pbkdf2::pbkdf2;
use pbkdf2::hmac::Hmac;
use sha2::Sha256;

fn encrypt(input_str: &[u8], dest: &mut [u8; 20]) {
    pbkdf2::<Hmac<Sha256>>(input_str, b"salt", 600_000, dest)
        .expect("HMAC can be initialized with any key length");
}

mod test {
    use super::*;

    #[test]
    fn test_encrypt() {
        let input_str = b"password";
        let dest = &mut [0u8; 20];
        encrypt(input_str, dest);
        assert_eq!(dest, &mut hex!("669cfe52482116fda1aa2cbe409b2f56c8e45637"));
    }
}
