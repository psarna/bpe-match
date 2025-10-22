use lazy_static::lazy_static;
use onig::Regex;

lazy_static! {
    // Regex to check if a character is a letter (matches \p{L})
    static ref LETTER_RE: Regex = Regex::new(r"\A\p{L}\z").unwrap();
    // Regex to check if a character is a number (matches \p{N})
    static ref NUMBER_RE: Regex = Regex::new(r"\A\p{N}\z").unwrap();
}

pub struct PatternIterator<'a> {
    text: &'a str,
    current_pos: usize,
}

impl<'a> PatternIterator<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            current_pos: 0,
        }
    }

    fn is_newline(c: char) -> bool {
        c == '\r' || c == '\n'
    }

    fn is_letter(c: char) -> bool {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        LETTER_RE.is_match(s)
    }

    fn is_number(c: char) -> bool {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        NUMBER_RE.is_match(s)
    }

    fn peek_char_at(&self, pos: usize) -> Option<char> {
        self.text[pos..].chars().next()
    }

    fn char_len_at(&self, pos: usize) -> usize {
        self.peek_char_at(pos).map(|c| c.len_utf8()).unwrap_or(0)
    }
}

impl<'a> Iterator for PatternIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.text.len() {
            return None;
        }

        let start_pos = self.current_pos;
        
        if let Some(end_pos) = self.try_match_apostrophe_contractions(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }
        
        if let Some(end_pos) = self.try_match_optional_nonalpha_plus_letters(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }
        
        if let Some(end_pos) = self.try_match_numbers_1_to_3(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }
        
        if let Some(end_pos) = self.try_match_space_plus_nonwhitespace_with_newlines(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }
        
        if let Some(end_pos) = self.try_match_whitespace_before_newlines(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }
        
        if let Some(end_pos) = self.try_match_whitespace_followed_by_whitespace_or_end(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }
        
        if let Some(end_pos) = self.try_match_any_whitespace(start_pos) {
            self.current_pos = end_pos;
            return Some(&self.text[start_pos..end_pos]);
        }

        self.current_pos = start_pos + self.char_len_at(start_pos).max(1);
        Some(&self.text[start_pos..self.current_pos])
    }
}

impl<'a> PatternIterator<'a> {
    fn try_match_apostrophe_contractions(&self, start_pos: usize) -> Option<usize> {
        if start_pos >= self.text.len() || !self.text[start_pos..].starts_with('\'') {
            return None;
        }

        let rest = &self.text[start_pos + 1..];
        let mut chars = rest.chars();
        
        if let Some(first_char) = chars.next() {
            if let Some(second_char) = chars.next() {
                let two_char_str = format!("{}{}", first_char, second_char);
                if two_char_str.eq_ignore_ascii_case("ll") || 
                   two_char_str.eq_ignore_ascii_case("ve") || 
                   two_char_str.eq_ignore_ascii_case("re") {
                    return Some(start_pos + 1 + first_char.len_utf8() + second_char.len_utf8());
                }
            }
            
            if first_char.to_ascii_lowercase() == 's' || 
               first_char.to_ascii_lowercase() == 'd' || 
               first_char.to_ascii_lowercase() == 'm' || 
               first_char.to_ascii_lowercase() == 't' {
                return Some(start_pos + 1 + first_char.len_utf8());
            }
        }
        
        None
    }

    fn try_match_optional_nonalpha_plus_letters(&self, start_pos: usize) -> Option<usize> {
        let mut pos = start_pos;

        // Optional non-alphabetic, non-numeric, non-newline character
        if let Some(c) = self.peek_char_at(pos) {
            if !Self::is_letter(c) && !Self::is_number(c) && !Self::is_newline(c) {
                pos += c.len_utf8();
            }
        }

        // Must be followed by one or more alphabetic characters
        let letter_start = pos;
        while let Some(c) = self.peek_char_at(pos) {
            if Self::is_letter(c) {
                pos += c.len_utf8();
            } else {
                break;
            }
        }

        // We need at least one letter after the optional non-alpha character
        if pos > letter_start {
            Some(pos)
        } else {
            None
        }
    }

    fn try_match_numbers_1_to_3(&self, start_pos: usize) -> Option<usize> {
        let mut pos = start_pos;
        let mut count = 0;
        
        while count < 3 {
            if let Some(c) = self.peek_char_at(pos) {
                if Self::is_number(c) {
                    pos += c.len_utf8();
                    count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if count > 0 {
            Some(pos)
        } else {
            None
        }
    }

    fn try_match_space_plus_nonwhitespace_with_newlines(&self, start_pos: usize) -> Option<usize> {
        if start_pos >= self.text.len() {
            return None;
        }
        
        let mut pos = start_pos;
        
        if let Some(c) = self.peek_char_at(pos) {
            if c == ' ' {
                pos += c.len_utf8();
            }
        }
        
        let special_start = pos;
        while let Some(c) = self.peek_char_at(pos) {
            if !c.is_whitespace() && !Self::is_letter(c) && !Self::is_number(c) {
                pos += c.len_utf8();
            } else {
                break;
            }
        }
        
        if pos > special_start {
            while let Some(c) = self.peek_char_at(pos) {
                if Self::is_newline(c) {
                    pos += c.len_utf8();
                } else {
                    break;
                }
            }
            Some(pos)
        } else {
            None
        }
    }

    fn try_match_whitespace_before_newlines(&self, start_pos: usize) -> Option<usize> {
        let mut pos = start_pos;
        
        while let Some(c) = self.peek_char_at(pos) {
            if c.is_whitespace() && !Self::is_newline(c) {
                pos += c.len_utf8();
            } else {
                break;
            }
        }
        
        let newline_start = pos;
        while let Some(c) = self.peek_char_at(pos) {
            if Self::is_newline(c) {
                pos += c.len_utf8();
            } else {
                break;
            }
        }
        
        if pos > newline_start {
            Some(pos)
        } else {
            None
        }
    }

    fn try_match_whitespace_followed_by_whitespace_or_end(&self, start_pos: usize) -> Option<usize> {
        if start_pos >= self.text.len() {
            return None;
        }
        
        if let Some(c) = self.peek_char_at(start_pos) {
            if !c.is_whitespace() || Self::is_newline(c) {
                return None;
            }
        }
        
        let mut positions = Vec::new();
        let mut pos = start_pos;
        
        while let Some(c) = self.peek_char_at(pos) {
            if c.is_whitespace() && !Self::is_newline(c) {
                pos += c.len_utf8();
                positions.push(pos);
            } else {
                break;
            }
        }
        
        for &end_pos in positions.iter().rev() {
            match self.peek_char_at(end_pos) {
                None => return Some(end_pos),
                Some(c) if c.is_whitespace() => return Some(end_pos),
                Some(_) => continue,
            }
        }
        
        None
    }

    fn try_match_any_whitespace(&self, start_pos: usize) -> Option<usize> {
        let mut pos = start_pos;
        let ws_start = pos;
        
        while let Some(c) = self.peek_char_at(pos) {
            if c.is_whitespace() {
                pos += c.len_utf8();
            } else {
                break;
            }
        }
        
        if pos > ws_start {
            Some(pos)
        } else {
            None
        }
    }
}

pub fn find_matches(text: &str) -> Vec<&str> {
    PatternIterator::new(text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use onig::Regex;

    const GPT4_PATTERN: &str = r"'(?i:[sdmt]|ll|ve|re)|[^\r\n\p{L}\p{N}]?+\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]++[\r\n]*|\s*[\r\n]|\s+(?!\S)|\s+";

    fn run_regex(text: &str) -> Vec<&str> {
        let re = Regex::new(GPT4_PATTERN).unwrap();
        re.find_iter(text).map(|(start, end)| &text[start..end]).collect()
    }

    #[test]
    fn test_simple_case() {
        let input = "¥hello";
        let regex_result = run_regex(input);
        let library_result = find_matches(input);

        println!("Input: {:?}", input);
        println!("Regex result: {:?}", regex_result);
        println!("Library result: {:?}", library_result);

        assert_eq!(regex_result, library_result);
    }

    #[test]
    fn test_unicode_case() {
        let input = "\u{115dc}¥";
        let regex_result = run_regex(input);
        let library_result = find_matches(input);

        println!("Input: {:?}", input);
        println!("Input chars:");
        for (i, c) in input.chars().enumerate() {
            println!("  [{}] '{}' - is_alphabetic: {}, is_numeric: {}, is_whitespace: {}",
                     i, c, c.is_alphabetic(), c.is_numeric(), c.is_whitespace());
        }
        println!("Regex result: {:?}", regex_result);
        println!("Library result: {:?}", library_result);

        assert_eq!(regex_result, library_result);
    }

    #[test]
    fn test_simple_letter() {
        let input = "hello";
        let regex_result = run_regex(input);
        let library_result = find_matches(input);

        println!("Input: {:?}", input);
        println!("Regex result: {:?}", regex_result);
        println!("Library result: {:?}", library_result);

        assert_eq!(regex_result, library_result);
    }

    proptest! {
        #[test]
        fn proptest_comparison(s in "\\PC*") {
            let regex_result = run_regex(&s);
            let library_result = find_matches(&s);

            assert_eq!(regex_result, library_result, "Mismatch found for input: {:?}", s);
        }
    }
}