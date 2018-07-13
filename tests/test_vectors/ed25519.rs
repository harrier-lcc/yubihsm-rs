use super::SignatureTestVector;

/// Ed25519 test vectors (from RFC 8032, converted to Rust bytestring literals)
pub const ED25519_TEST_VECTORS: &[SignatureTestVector] = &[
    SignatureTestVector {
        sk: b"\x9D\x61\xB1\x9D\xEF\xFD\x5A\x60\xBA\x84\x4A\xF4\x92\xEC\x2C\xC4\x44\x49\xC5\x69\x7B\x32\x69\x19\x70\x3B\xAC\x03\x1C\xAE\x7F\x60",
        pk: b"\xD7\x5A\x98\x01\x82\xB1\x0A\xB7\xD5\x4B\xFE\xD3\xC9\x64\x07\x3A\x0E\xE1\x72\xF3\xDA\xA6\x23\x25\xAF\x02\x1A\x68\xF7\x07\x51\x1A",
        msg: b"",
        sig: b"\xE5\x56\x43\x00\xC3\x60\xAC\x72\x90\x86\xE2\xCC\x80\x6E\x82\x8A\x84\x87\x7F\x1E\xB8\xE5\xD9\x74\xD8\x73\xE0\x65\x22\x49\x01\x55\x5F\xB8\x82\x15\x90\xA3\x3B\xAC\xC6\x1E\x39\x70\x1C\xF9\xB4\x6B\xD2\x5B\xF5\xF0\x59\x5B\xBE\x24\x65\x51\x41\x43\x8E\x7A\x10\x0B",
    },
    SignatureTestVector {
        sk: b"\x4C\xCD\x08\x9B\x28\xFF\x96\xDA\x9D\xB6\xC3\x46\xEC\x11\x4E\x0F\x5B\x8A\x31\x9F\x35\xAB\xA6\x24\xDA\x8C\xF6\xED\x4F\xB8\xA6\xFB",
        pk: b"\x3D\x40\x17\xC3\xE8\x43\x89\x5A\x92\xB7\x0A\xA7\x4D\x1B\x7E\xBC\x9C\x98\x2C\xCF\x2E\xC4\x96\x8C\xC0\xCD\x55\xF1\x2A\xF4\x66\x0C",
        msg: b"\x72",
        sig: b"\x92\xA0\x09\xA9\xF0\xD4\xCA\xB8\x72\x0E\x82\x0B\x5F\x64\x25\x40\xA2\xB2\x7B\x54\x16\x50\x3F\x8F\xB3\x76\x22\x23\xEB\xDB\x69\xDA\x08\x5A\xC1\xE4\x3E\x15\x99\x6E\x45\x8F\x36\x13\xD0\xF1\x1D\x8C\x38\x7B\x2E\xAE\xB4\x30\x2A\xEE\xB0\x0D\x29\x16\x12\xBB\x0C\x00",
    },
    SignatureTestVector {
        sk: b"\xC5\xAA\x8D\xF4\x3F\x9F\x83\x7B\xED\xB7\x44\x2F\x31\xDC\xB7\xB1\x66\xD3\x85\x35\x07\x6F\x09\x4B\x85\xCE\x3A\x2E\x0B\x44\x58\xF7",
        pk: b"\xFC\x51\xCD\x8E\x62\x18\xA1\xA3\x8D\xA4\x7E\xD0\x02\x30\xF0\x58\x08\x16\xED\x13\xBA\x33\x03\xAC\x5D\xEB\x91\x15\x48\x90\x80\x25",
        msg: b"\xAF\x82",
        sig: b"\x62\x91\xD6\x57\xDE\xEC\x24\x02\x48\x27\xE6\x9C\x3A\xBE\x01\xA3\x0C\xE5\x48\xA2\x84\x74\x3A\x44\x5E\x36\x80\xD7\xDB\x5A\xC3\xAC\x18\xFF\x9B\x53\x8D\x16\xF2\x90\xAE\x67\xF7\x60\x98\x4D\xC6\x59\x4A\x7C\x15\xE9\x71\x6E\xD2\x8D\xC0\x27\xBE\xCE\xEA\x1E\xC4\x0A",
    }
];
