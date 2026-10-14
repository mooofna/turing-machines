//! Tokens of the description language.
//!
//! The lexer turns a whole source text into a flat list of tokens with byte spans. It never
//! fails: an unterminated string or quote becomes a [`TokenKind::Error`] token, which the
//! parser reports before it skips to the next line.
//!
//! Whether a token is a state name, a symbol or a keyword depends on where it stands, so words
//! stay generic here. A one-character word such as `x` can be a symbol, a longer word such as
//! `q0` only a state name or a keyword.

use alloc::string::String;
use alloc::vec::Vec;

use crate::diag::Span;

/// A token with its source span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Token<'a> {
    pub(super) kind: TokenKind<'a>,
    pub(super) span: Span,
}

/// What a token is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TokenKind<'a> {
    /// A letter or `_`, followed by letters, digits, `_` and `'`.
    Word(&'a str),
    /// A run of ASCII digits.
    Number(&'a str),
    /// A quoted symbol such as `'*'`.
    Quoted(char),
    /// A string literal with its escapes resolved.
    Str(String),
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `,`
    Comma,
    /// `=`
    Eq,
    /// `*`
    Star,
    /// Any other single character, for example `#` or `␣`.
    Other(char),
    /// `//` and the rest of the line. The text excludes the slashes and trailing whitespace.
    Comment(&'a str),
    /// The end of a line.
    Newline,
    /// Text that is not a token.
    Error(LexError),
}

/// Why a piece of text is not a token.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum LexError {
    /// A string literal runs to the end of the line.
    UnterminatedString,
    /// A `'` that is not followed by one character and a closing `'`.
    UnterminatedQuote,
}

/// Whether `c` can start a word.
pub(super) fn is_word_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Whether `c` can continue a word.
pub(super) fn is_word_continue(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit() || c == '_' || c == '\''
}

/// Converts a byte offset to the `u32` used by [`Span`]. Offsets beyond 4 GiB saturate.
pub(super) fn offset(pos: usize) -> u32 {
    u32::try_from(pos).unwrap_or(u32::MAX)
}

/// Splits `source` into tokens. The last token is always a [`TokenKind::Newline`], so every
/// line, the last one included, is terminated.
pub(super) fn tokenize(source: &str) -> Vec<Token<'_>> {
    let mut lexer = Lexer {
        source,
        pos: 0,
        tokens: Vec::new(),
    };
    lexer.run();
    lexer.tokens
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    fn rest(&self) -> &'a str {
        &self.source[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    /// The length of the rest of the current line, without the line break.
    fn line_len(&self) -> usize {
        self.rest().find('\n').unwrap_or(self.rest().len())
    }

    fn push(&mut self, kind: TokenKind<'a>, start: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(offset(start), offset(self.pos)),
        });
    }

    fn run(&mut self) {
        while let Some(c) = self.peek() {
            let start = self.pos;
            if c == '\n' {
                self.pos += 1;
                self.push(TokenKind::Newline, start);
            } else if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else if self.rest().starts_with("//") {
                self.comment();
            } else if is_word_start(c) {
                self.pos += self
                    .rest()
                    .find(|c| !is_word_continue(c))
                    .unwrap_or(self.rest().len());
                self.push(TokenKind::Word(&self.source[start..self.pos]), start);
            } else if c.is_ascii_digit() {
                self.pos += self
                    .rest()
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(self.rest().len());
                self.push(TokenKind::Number(&self.source[start..self.pos]), start);
            } else if c == '"' {
                self.string();
            } else if c == '\'' {
                self.quoted();
            } else {
                self.pos += c.len_utf8();
                let kind = match c {
                    '(' => TokenKind::LParen,
                    ')' => TokenKind::RParen,
                    '{' => TokenKind::LBrace,
                    '}' => TokenKind::RBrace,
                    ',' => TokenKind::Comma,
                    '=' => TokenKind::Eq,
                    '*' => TokenKind::Star,
                    other => TokenKind::Other(other),
                };
                self.push(kind, start);
            }
        }
        self.push(TokenKind::Newline, self.pos);
    }

    /// `//` up to the end of the line.
    fn comment(&mut self) {
        let start = self.pos;
        let len = self.line_len();
        let text = self.source[start + 2..start + len].trim_end();
        self.pos += 2 + text.len();
        self.push(TokenKind::Comment(text), start);
        self.pos = start + len;
    }

    /// A string literal. `\"`, `\\`, `\n` and `\t` are escapes; any other backslash stands
    /// for itself.
    fn string(&mut self) {
        let start = self.pos;
        let mut value = String::new();
        let mut chars = self.rest()[1..].char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => {
                    self.pos += i + 2;
                    self.push(TokenKind::Str(value), start);
                    return;
                }
                '\n' => break,
                '\\' => match chars.clone().next() {
                    Some((_, e @ ('"' | '\\' | 'n' | 't'))) => {
                        chars.next();
                        value.push(match e {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                    }
                    _ => value.push('\\'),
                },
                other => value.push(other),
            }
        }
        // Unterminated: the error covers the rest of the line without trailing whitespace.
        let len = self.line_len();
        let text = self.rest()[..len].trim_end();
        self.pos += text.len();
        self.push(TokenKind::Error(LexError::UnterminatedString), start);
        self.pos = start + len;
    }

    /// A quoted symbol: `'`, exactly one character other than a line break, `'`.
    fn quoted(&mut self) {
        let start = self.pos;
        let mut chars = self.rest()[1..].chars();
        match (chars.next(), chars.next()) {
            (Some(c), Some('\'')) if c != '\n' => {
                self.pos += 2 + c.len_utf8();
                self.push(TokenKind::Quoted(c), start);
            }
            _ => {
                self.pos += 1;
                self.push(TokenKind::Error(LexError::UnterminatedQuote), start);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind<'_>> {
        tokenize(source).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn transition_line() {
        use TokenKind::*;
        assert_eq!(
            kinds("δ(q0, #) = (q1, ␣, R) // done "),
            [
                Word("δ"),
                LParen,
                Word("q0"),
                Comma,
                Other('#'),
                RParen,
                Eq,
                LParen,
                Word("q1"),
                Comma,
                Other('␣'),
                Comma,
                Word("R"),
                RParen,
                Comment(" done"),
                Newline,
            ]
        );
    }

    #[test]
    fn spans_are_byte_offsets() {
        let tokens = tokenize("δ(q') // x");
        assert_eq!(tokens[0].span, Span::new(0, 2));
        assert_eq!(tokens[1].span, Span::new(2, 3));
        assert_eq!(tokens[2].kind, TokenKind::Word("q'"));
        assert_eq!(tokens[2].span, Span::new(3, 5));
        assert_eq!(tokens[4].span, Span::new(7, 11));
    }

    #[test]
    fn strings_and_escapes() {
        assert_eq!(
            kinds(r#""a\"b\\c\n\q""#)[0],
            TokenKind::Str("a\"b\\c\n\\q".into())
        );
        let tokens = tokenize("input \"abc \r\nkind");
        assert_eq!(
            tokens[1].kind,
            TokenKind::Error(LexError::UnterminatedString)
        );
        assert_eq!(tokens[1].span, Span::new(6, 10));
        assert_eq!(tokens[2].kind, TokenKind::Newline);
        assert_eq!(tokens[3].kind, TokenKind::Word("kind"));
    }

    #[test]
    fn quoted_symbols() {
        assert_eq!(
            kinds("'*' '''")[..2],
            [TokenKind::Quoted('*'), TokenKind::Quoted('\'')]
        );
        assert_eq!(
            kinds("'ab'")[0],
            TokenKind::Error(LexError::UnterminatedQuote)
        );
        assert_eq!(kinds("'")[0], TokenKind::Error(LexError::UnterminatedQuote));
        assert_eq!(
            kinds("'\n'")[0],
            TokenKind::Error(LexError::UnterminatedQuote)
        );
    }

    #[test]
    fn numbers_and_comments() {
        use TokenKind::*;
        assert_eq!(
            kinds("tapes = 12//x\r\n"),
            [
                Word("tapes"),
                Eq,
                Number("12"),
                Comment("x"),
                Newline,
                Newline
            ]
        );
    }
}
