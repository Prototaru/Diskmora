//! Byte signatures and format codes used by the scanner. No extension fallback.

pub const FORMATS: &[&str] = &["UNKNOWN", "PNG", "JPEG", "GIF", "PDF", "ZIP", "RAR", "MP3", "EXE", "ELF", "WEBP", "BMP", "MP4", "M4A", "FLAC", "MOV", "HEIC"];

pub fn detect(data: &[u8]) -> &'static str {
    if data.len() < 4 { return "UNKNOWN"; }
    match data {
        [0x89, 0x50, 0x4e, 0x47, ..] => "PNG",
        [0xff, 0xd8, 0xff, ..] => "JPEG",
        [0x47, 0x49, 0x46, 0x38, ..] => "GIF",
        [0x25, 0x50, 0x44, 0x46, ..] => "PDF",
        [0x50, 0x4b, 0x03, 0x04, ..] => "ZIP",
        [0x52, 0x61, 0x72, 0x21, ..] => "RAR",
        [0x49, 0x44, 0x33, ..] => "MP3",
        [0x4d, 0x5a, ..] => "EXE",
        [0x7f, 0x45, 0x4c, 0x46, ..] => "ELF",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "WEBP",
        [0x42, 0x4d, ..] => "BMP",
        [_, _, _, _, b'f', b't', b'y', b'p', a, b, c, d, ..] => iso_brand([*a, *b, *c, *d]),
        [0xff, 0xfb, ..] => "MP3",
        [b'f', b'L', b'a', b'C', ..] => "FLAC",
        _ => "UNKNOWN",
    }
}

fn iso_brand(brand: [u8; 4]) -> &'static str {
    match &brand {
        b"qt  " => "MOV",
        b"isom" | b"iso2" | b"iso3" | b"iso4" | b"iso5" | b"iso6" | b"mp41" | b"mp42" | b"avc1" | b"M4V "
        | b"M4VH" | b"M4VP" | b"MSNV" | b"dash" | b"f4v " | b"3gp4" | b"3gp5" | b"3gp6" | b"3g2a" => "MP4",
        b"M4A " | b"M4B " => "M4A",
        b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx" | b"mif1" | b"msf1" => "HEIC",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_original_signatures_and_codes() {
        assert_eq!(FORMATS[1], "PNG"); assert_eq!(FORMATS[12], "MP4"); assert_eq!(FORMATS[16], "HEIC");
        for (head, expected) in [
            (b"\x89PNG".as_slice(), "PNG"), (b"\xff\xd8\xff\0", "JPEG"), (b"GIF89a", "GIF"),
            (b"%PDF-1.7", "PDF"), (b"PK\x03\x04", "ZIP"), (b"Rar!", "RAR"), (b"ID3\0", "MP3"),
            (b"MZ\0\0", "EXE"), (b"\x7fELF", "ELF"), (b"RIFF\0\0\0\0WEBP", "WEBP"),
            (b"BM\0\0", "BMP"), (b"fLaC", "FLAC"), (b"\0\0\0\x18ftypqt  ", "MOV"),
            (b"\0\0\0\x18ftypM4A ", "M4A"), (b"\0\0\0\x18ftypmif1", "HEIC"), (b"", "UNKNOWN"),
        ] { assert_eq!(detect(head), expected); }
    }
}
