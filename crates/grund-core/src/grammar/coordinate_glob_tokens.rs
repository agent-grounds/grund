//! The body of a coordinate glob read into tokens in one pass, over the operator table
//! the citation guard reads (§FS-coordinate-globs.grammar.operators), refusing at the
//! first bracket or escape the grammar cannot close (§FS-coordinate-globs.errors.reasons).

use super::coordinate_glob::{GlobRefusal, GlobStage, Spanned, Token, offered, refusal};
use super::glob_candidate::GlobOperator;

/// The body's reader: one pass, refusing at the first bracket or escape the
/// grammar cannot close (§FS-coordinate-globs.grammar.operators, §FS-coordinate-globs.grammar.braces).
pub(super) struct Tokenizer<'t> {
    text: &'t str,
    chars: Vec<(usize, char)>,
    at: usize,
    /// The stage an accepted glob must pass, or `None` to offer nothing.
    offer: Option<GlobStage<'t>>,
    tokens: Vec<Spanned>,
    group: Option<usize>,
}

impl<'t> Tokenizer<'t> {
    pub(super) fn new(text: &'t str, body: usize, offer: Option<GlobStage<'t>>) -> Self {
        let chars = text[body..]
            .char_indices()
            .map(|(at, ch)| (body + at, ch))
            .collect();
        Self {
            text,
            chars,
            at: 0,
            offer,
            tokens: Vec::new(),
            group: None,
        }
    }

    fn refuse(&self, at: usize, reason: &str, accepted: Vec<String>) -> GlobRefusal {
        refusal(self.text, at, reason, accepted)
    }

    fn next(&mut self) -> Option<(usize, char)> {
        let next = self.chars.get(self.at).copied();
        self.at += usize::from(next.is_some());
        next
    }

    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.at + ahead).map(|&(_, ch)| ch)
    }

    fn offset(&self) -> usize {
        self.chars
            .get(self.at)
            .map_or(self.text.len(), |&(at, _)| at)
    }

    /// The character after a `\`, or the refusal of a `\` that ends the pattern.
    fn escaped(&mut self, at: usize) -> Result<char, GlobRefusal> {
        self.next()
            .map(|(_, ch)| ch)
            .ok_or_else(|| self.refuse(at, "\"\\\" escapes nothing", Vec::new()))
    }

    pub(super) fn run(mut self) -> Result<Vec<Spanned>, GlobRefusal> {
        while let Some((start, ch)) = self.next() {
            let token = match GlobOperator::of(ch) {
                Some(GlobOperator::Star) => {
                    let mut run = 1;
                    while self.peek(0) == Some('*') {
                        self.next();
                        run += 1;
                    }
                    Token::Stars(run)
                }
                Some(GlobOperator::Question) => Token::Any,
                Some(GlobOperator::Class) => self.class(start)?,
                Some(GlobOperator::Group) => self.open(start)?,
                None => match ch {
                    '\\' => Token::Literal {
                        ch: self.escaped(start)?,
                        escaped: true,
                    },
                    ',' if self.group.is_some() => self.comma(),
                    '}' if self.group.is_some() => self.close()?,
                    '.' => Token::Dot,
                    ch => Token::Literal { ch, escaped: false },
                },
            };
            let end = self.offset();
            self.tokens.push(Spanned { token, start, end });
        }
        if let Some(open) = self.group {
            return Err(self.refuse(self.tokens[open].start, "\"{\" is never closed", Vec::new()));
        }
        Ok(self.tokens)
    }

    fn open(&mut self, start: usize) -> Result<Token, GlobRefusal> {
        if self.group.is_some() {
            return Err(self.refuse(start, "braces do not nest", Vec::new()));
        }
        self.group = Some(self.tokens.len());
        Ok(Token::Open {
            alternatives: vec![self.tokens.len() + 1],
            close: 0,
        })
    }

    fn comma(&mut self) -> Token {
        let next = self.tokens.len() + 1;
        if let Some(Token::Open { alternatives, .. }) =
            self.group.map(|open| &mut self.tokens[open].token)
        {
            alternatives.push(next);
        }
        Token::Comma { close: 0 }
    }

    /// Close the group, pointing its `{` and commas at this `}`, and refuse a
    /// `{N..M}` range (§FS-coordinate-globs.grammar.braces).
    fn close(&mut self) -> Result<Token, GlobRefusal> {
        let open = self.group.take().expect("guarded by the caller");
        let close = self.tokens.len();
        let mut single = true;
        for spanned in &mut self.tokens[open + 1..] {
            if let Token::Comma { close: target } = &mut spanned.token {
                *target = close;
                single = false;
            }
        }
        let start = self.tokens[open].start;
        if let Token::Open { close: target, .. } = &mut self.tokens[open].token {
            *target = close;
        }
        let range = self.tokens[open + 1..]
            .windows(2)
            .any(|pair| pair[0].token == Token::Dot && pair[1].token == Token::Dot);
        if single && range {
            return Err(self.refuse(
                start,
                "braces take no range; list each alternative",
                Vec::new(),
            ));
        }
        Ok(Token::Close)
    }

    /// One class member, its escape read.
    fn member(&mut self) -> Result<Option<(usize, char)>, GlobRefusal> {
        let Some((at, ch)) = self.next() else {
            return Ok(None);
        };
        let ch = if ch == '\\' { self.escaped(at)? } else { ch };
        Ok(Some((at, ch)))
    }

    /// `[…]`, `[!…]` or `[^…]`: a leading `]` and a leading or trailing `-` are
    /// members, and a range must run forwards (§FS-coordinate-globs.grammar.operators).
    fn class(&mut self, open: usize) -> Result<Token, GlobRefusal> {
        let negated = matches!(self.peek(0), Some('!' | '^'));
        if negated {
            self.next();
        }
        let unclosed = |this: &Self| this.refuse(open, "\"[\" is never closed", Vec::new());
        let mut members = Vec::new();
        loop {
            if self.peek(0) == Some(']') && !members.is_empty() {
                self.next();
                return Ok(Token::Class { negated, members });
            }
            let Some((low_at, low)) = self.member()? else {
                return Err(unclosed(self));
            };
            if self.peek(0) != Some('-') || matches!(self.peek(1), None | Some(']')) {
                members.push((low, low));
                continue;
            }
            let (dash, _) = self.next().expect("peeked");
            let Some((_, high)) = self.member()? else {
                return Err(unclosed(self));
            };
            if high < low {
                let end = self.offset();
                let (low_text, high_text) = (&self.text[low_at..dash], &self.text[dash + 1..end]);
                let rewrite = format!(
                    "{}{high_text}-{low_text}{}",
                    &self.text[..low_at],
                    &self.text[end..]
                );
                let reason = format!("the range \"{}\" runs backwards", &self.text[low_at..end]);
                return Err(self.refuse(low_at, &reason, offered(self.offer, [rewrite])));
            }
            members.push((low, high));
        }
    }
}
