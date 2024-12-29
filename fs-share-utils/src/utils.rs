use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

#[inline]
fn download_dir() -> Option<PathBuf> {
    #[cfg(not(target_os = "android"))]
    return dirs::download_dir();
    #[cfg(target_os = "android")]
    {
        let dir1 = PathBuf::from("/storage/emulated/0/Download");
        if dir1.is_dir() {
            return Some(dir1);
        };
        let dir2 = PathBuf::from("/sdcard/Download");
        if dir2.is_dir() {
            Some(dir2)
        } else {
            None
        }
    }
}

pub fn create_file_path<T: AsRef<str>>(value: T) -> PathBuf {
    let download = download_dir().unwrap_or(PathBuf::from("."));
    let file_path = PathBuf::from_iter([download, PathBuf::from(value.as_ref())].iter());
    if !file_path.is_file() {
        return file_path;
    }
    let dir = file_path.parent().unwrap_or(Path::new("."));
    let ext = file_path.extension().unwrap_or_default();
    let name = file_path.file_stem().unwrap_or(OsStr::new("unknown"));
    let mut i: u32 = 1;
    loop {
        let mut new_name = OsString::from(name);
        new_name.push(format!("_{}", i));
        new_name.push(ext);
        let new_path = dir.join(new_name);
        if !new_path.is_file() {
            return new_path;
        }
        i += 1;
    }
}

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
