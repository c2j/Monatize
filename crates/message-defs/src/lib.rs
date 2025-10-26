// Re-export generated types
#![allow(non_snake_case, non_camel_case_types)]

pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub use generated::*;

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn round_trip_http() {
        let req = HttpRequest { url: "https://example.com".into() };
        let bytes = bincode::serialize(&req).unwrap();
        let de: HttpRequest = bincode::deserialize(&bytes).unwrap();
        assert_eq!(req, de);

        let resp = HttpResponse { status: 200, headers: vec![("Content-Type".into(), "text/plain".into())], body: bytes::Bytes::from_static(b"hello") };
        let bytes = bincode::serialize(&resp).unwrap();
        let de: HttpResponse = bincode::deserialize(&bytes).unwrap();
        assert_eq!(resp, de);
    }

    #[test]
    fn idl_hash_stable() {
        // Compute hash of IDL content in the same way as build.rs (concatenated with trailing newline)
        let mut s = String::from(include_str!("../idl/core.idl"));
        s.push('\n');
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let hash = hasher.finalize();
        let hex = format!("{:x}", hash);
        assert_eq!(hex, GEN_HASH, "IDL hash mismatch; regenerate needed");
    }
}

