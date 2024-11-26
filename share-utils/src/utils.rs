use sha2::{Digest, Sha256};

pub fn sha256<T: AsRef<[u8]>>(value: T) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use crate::sha256;

    #[test]
    fn test_sha256_str() {
        let vec_to_hex =
            |v: Vec<u8>| -> String { v.iter().map(|u| format!("{:02x}", u)).collect() };
        let input = "";
        let got = sha256(input.as_bytes());
        assert_eq!(got.len(), 32);
        let got_hex = vec_to_hex(got);
        assert_eq!(
            got_hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned()
        );
        let input = "abc";
        let got = vec_to_hex(sha256(input.as_bytes()));
        assert_eq!(
            got,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned()
        );
        let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let got = vec_to_hex(sha256(input.as_bytes()));
        assert_eq!(
            got,
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1".to_owned()
        );
        let input = "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
        let got = vec_to_hex(sha256(input.as_bytes()));
        assert_eq!(
            got,
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1".to_owned()
        );
    }
}
