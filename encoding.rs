// This file is based on section 13.2 of the HTML5 specification
// The goal is to implement the encoding sniffing algorithm (13.2.3.2)
// https://html.spec.whatwg.org/multipage/parsing.html#determining-the-character-encoding

/// When the HTML parser is decoding an input byte stream, it uses a character encoding and a confidence. The confidence is either tentative, certain, or irrelevant. The encoding used, and whether the confidence in that encoding is tentative or certain, is used during the parsing to determine whether to change the encoding. If no encoding is necessary, e.g. because the parser is operating on a Unicode stream and doesn't have to use a character encoding at all, then the confidence is irrelevant.
///
/// https://html.spec.whatwg.org/multipage/parsing.html#the-input-byte-stream
pub enum Confidence {
    Certain,
    Tentative,
    Irrelevant,
}

/// User agents must support the encodings defined in Encoding, including, but not limited to, UTF-8, ISO-8859-2, ISO-8859-7, ISO-8859-8, windows-874, windows-1250, windows-1251, windows-1252, windows-1254, windows-1255, windows-1256, windows-1257, windows-1258, GBK, Big5, ISO-2022-JP, Shift_JIS, EUC-KR, UTF-16BE, UTF-16LE, UTF-16BE/LE, and x-user-defined. User agents must not support other encodings.
///
/// https://html.spec.whatwg.org/multipage/parsing.html#character-encodings
pub enum Encoding {
    Utf8,
    Utf16,
}
