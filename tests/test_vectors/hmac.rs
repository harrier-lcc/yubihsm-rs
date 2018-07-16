use super::HMACTestVector;

/// HMAC-SHA-256 test vectors (from RFC 4231, converted to Rust bytestring literals)
pub const HMAC_SHA256_TEST_VECTORS: &[HMACTestVector] = &[
    HMACTestVector {
        key: b"\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B\x0B",
        msg: b"\x48\x69\x20\x54\x68\x65\x72\x65",
        tag: b"\xB0\x34\x4C\x61\xD8\xDB\x38\x53\x5C\xA8\xAF\xCE\xAF\x0B\xF1\x2B\x88\x1D\xC2\x00\xC9\x83\x3D\xA7\x26\xE9\x37\x6C\x2E\x32\xCF\xF7"
    },
    HMACTestVector {
        key: b"\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA\xAA",
        msg: b"\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD\xDD",
        tag: b"\x77\x3E\xA9\x1E\x36\x80\x0E\x46\x85\x4D\xB8\xEB\xD0\x91\x81\xA7\x29\x59\x09\x8B\x3E\xF8\xC1\x22\xD9\x63\x55\x14\xCE\xD5\x65\xFE"
    },
    HMACTestVector {
        key: b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19",
        msg: b"\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD\xCD",
        tag: b"\x82\x55\x8A\x38\x9A\x44\x3C\x0E\xA4\xCC\x81\x98\x99\xF2\x08\x3A\x85\xF0\xFA\xA3\xE5\x78\xF8\x07\x7A\x2E\x3F\xF4\x67\x29\x66\x5B"
    },
];
