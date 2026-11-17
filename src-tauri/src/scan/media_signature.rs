use super::base_signature;

pub const HEADER_BYTES: usize = 4096;
pub const EXTRA_FORMATS: &[&str] = &["MKV", "WEBM", "AVI", "FLV", "MPEG", "TS", "WAV", "AAC", "OGG", "OPUS", "AIFF", "AMR", "AC3", "APE", "M2TS", "3GP", "WMA", "WMV", "ASF"];

pub fn detect(data: &[u8]) -> String {
    let has = |needle: &[u8]| data.windows(needle.len()).any(|part| part == needle);
    let extended = match data {
        [0x1a, 0x45, 0xdf, 0xa3, ..] if has(b"\x42\x82\x84webm") => Some("WEBM"),
        [0x1a, 0x45, 0xdf, 0xa3, ..] if has(b"\x42\x82\x88matroska") => Some("MKV"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'A', b'V', b'I', b' ', ..] => Some("AVI"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'A', b'V', b'E', ..] => Some("WAV"),
        [b'F', b'L', b'V', 1, ..] => Some("FLV"),
        [0, 0, 1, 0xba | 0xb3, ..] => Some("MPEG"),
        [b'F', b'O', b'R', b'M', _, _, _, _, b'A', b'I', b'F', b'F' | b'C', ..] => Some("AIFF"),
        [b'O', b'g', b'g', b'S', 0, ..] if data.len() >= 28 => {
            let packet = data.get(27 + data[26] as usize..).unwrap_or_default();
            if packet.starts_with(b"OpusHead") { Some("OPUS") } else { Some("OGG") }
        }
        [b'#', b'!', b'A', b'M', b'R', b'\n', ..] | [b'#', b'!', b'A', b'M', b'R', b'-', b'W', b'B', b'\n', ..] => Some("AMR"),
        [0x0b, 0x77, ..] => Some("AC3"),
        [b'M', b'A', b'C', b' ', ..] => Some("APE"),
        [0xff, second, third, ..] if second & 0xf6 == 0xf0 && (third >> 2) & 15 != 15 => Some("AAC"),
        [_, _, _, _, b'f', b't', b'y', b'p', b'3', b'g', b'p' | b'2', _, ..] => Some("3GP"),
        _ if data.len() > 376 && [0, 188, 376].iter().all(|index| data[*index] == 0x47) => Some("TS"),
        _ if data.len() > 388 && [4, 196, 388].iter().all(|index| data[*index] == 0x47) => Some("M2TS"),
        _ if data.starts_with(b"\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c") => {
            if has(b"\xc0\xef\x19\xbc\x4d\x5b\xcf\x11\xa8\xfd\x00\x80\x5f\x5c\x44\x2b") { Some("WMV") }
            else if has(b"\x40\x9e\x69\xf8\x4d\x5b\xcf\x11\xa8\xfd\x00\x80\x5f\x5c\x44\x2b") { Some("WMA") }
            else { Some("ASF") }
        }
        _ => None,
    };
    extended.map(str::to_owned).unwrap_or_else(|| base_signature::detect(data).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_media_containers_without_filename_hints() {
        let samples: &[(&[u8], &str)] = &[
            (b"\x1a\x45\xdf\xa3\x42\x82\x84webm", "WEBM"), (b"\x1a\x45\xdf\xa3\x42\x82\x88matroska", "MKV"),
            (b"RIFF\0\0\0\0AVI ", "AVI"), (b"RIFF\0\0\0\0WAVE", "WAV"), (b"FLV\x01", "FLV"),
            (b"FORM\0\0\0\0AIFF", "AIFF"), (b"#!AMR\n", "AMR"), (b"MAC \0\0", "APE"),
            (b"\0\0\x01\xba", "MPEG"), (b"\x0b\x77\0\0", "AC3"), (b"\xff\xf1\x50\0", "AAC"),
            (b"\0\0\0\x18ftyp3gp6", "3GP"), (b"\0\0\0\x18ftypisom", "MP4"),
        ];
        for (head, expected) in samples { assert_eq!(detect(head), *expected); }
        let mut ogg = b"OggS".to_vec(); ogg.resize(28, 0); ogg[26] = 1;
        assert_eq!(detect(&ogg), "OGG"); ogg.extend_from_slice(b"OpusHead"); assert_eq!(detect(&ogg), "OPUS");
        for (stride, offset, expected) in [(188, 0, "TS"), (192, 4, "M2TS")] {
            let mut head = vec![0; stride * 3]; for index in 0..3 { head[offset + stride * index] = 0x47; }
            assert_eq!(detect(&head), expected);
        }
        for head in [b"".as_slice(), b"RIFF", b"\x1a\x45\xdf\xa3", b"\xff\xf1", b"OggS"] { assert_eq!(detect(head), "UNKNOWN"); }
    }
}
