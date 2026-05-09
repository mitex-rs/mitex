use std::fs;
use std::path::PathBuf;

use super::prelude::*;

const CASE: &str = "source-tex/arxiv.2301.00029/YM_sym";

#[test]
#[ignore = "requires local/mitex-testset checkout"]
fn test_ym_sym_typ_hash() {
    let root = workspace_root();
    let input_path = root.join("local/mitex-testset").join(format!("{CASE}.tex"));
    let output_path = root.join("local/mitex-output").join(format!("{CASE}.typ"));
    let hash_path = root
        .join("fixtures/mitex-testset-hashes")
        .join(format!("{CASE}.typ.sha256"));

    assert!(
        input_path.exists(),
        "missing test input: {}\nrun: git clone --depth 1 https://github.com/mitex-rs/mitex-testset/ local/mitex-testset",
        input_path.display()
    );

    let input = fs::read_to_string(&input_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", input_path.display()));
    let typ = convert_text(&input).unwrap_or_else(|err| panic!("failed to convert {CASE}: {err}"));
    let actual_hash = sha256_hex(typ.as_bytes());

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }
    fs::write(&output_path, &typ)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", output_path.display()));

    let expected_hash = fs::read_to_string(&hash_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", hash_path.display()));
    assert_eq!(actual_hash, expected_hash.trim());
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate manifest should be under crates/mitex")
        .to_path_buf()
}

fn sha256_hex(input: &[u8]) -> String {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut data = input.to_vec();
    let bit_len = (data.len() as u64) * 8;
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = H0;
    for chunk in data.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (word, bytes) in w.iter_mut().zip(chunk.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        for (dst, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *dst = dst.wrapping_add(value);
        }
    }

    h.iter().map(|word| format!("{word:08x}")).collect()
}
