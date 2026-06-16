#[derive(Debug, PartialEq)] // word part regarding the echo"hello" and echo "hello"
pub enum WordPart {
    Word(String),
    SingleQuoted(String),
    DoubleQuoted(String),
}

#[derive(Debug, PartialEq)] // for assert_eq!, derive system traits  // each variant represents part of the command
pub enum Token {
    Word(Vec<WordPart>),
    Pipe,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
    And,
    Or,
}

// error
#[derive(Debug, PartialEq)]
pub enum TokenizeError {
    UnclosedQuote,
    InvalidOperator(String),
    BadSyntax(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut token_vec = Vec::new();
    let mut token_string = input.chars().peekable();
    let mut string_buffer: String = String::new();
    let mut word_buffer = Vec::new();
    let mut unquoted = false;

    while let Some(character) = token_string.next() {
        match character {
            ' ' | '\t' | '\n' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }
                if !word_buffer.is_empty() {
                    unquoted = false;
                    token_vec.push(Token::Word(std::mem::take(&mut word_buffer)));
                }
            }
            '\'' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }

                let mut single_quotes = true;

                while let Some(ch) = token_string.next() {
                    if ch == '\'' {
                        single_quotes = false;
                        break;
                    } else {
                        string_buffer.push(ch);
                    }
                }

                if single_quotes {
                    return Err(TokenizeError::UnclosedQuote);
                }

                word_buffer.push(WordPart::SingleQuoted(std::mem::take(&mut string_buffer)));
            }
            '"' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }

                let mut double_quotes = true;

                while let Some(ch) = token_string.next() {
                    if ch == '"' {
                        double_quotes = false;
                        break;
                    } else {
                        string_buffer.push(ch);
                    }
                }

                if double_quotes {
                    return Err(TokenizeError::UnclosedQuote);
                }

                word_buffer.push(WordPart::DoubleQuoted(std::mem::take(&mut string_buffer)));
            }
            '|' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }
                if !word_buffer.is_empty() {
                    unquoted = false;
                    token_vec.push(Token::Word(std::mem::take(&mut word_buffer)));
                }

                if token_string.peek() == Some(&'|') {
                    token_string.next();
                    token_vec.push(Token::Or);
                } else {
                    token_vec.push(Token::Pipe);
                }
            }
            '&' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }
                if !word_buffer.is_empty() {
                    unquoted = false;
                    token_vec.push(Token::Word(std::mem::take(&mut word_buffer)));
                }

                if token_string.peek() == Some(&'&') {
                    token_string.next();
                    token_vec.push(Token::And);
                } else {
                    return Err(TokenizeError::InvalidOperator(character.to_string()));
                }
            }
            '>' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }
                if !word_buffer.is_empty() {
                    unquoted = false;
                    token_vec.push(Token::Word(std::mem::take(&mut word_buffer)));
                }

                if token_string.peek() == Some(&'>') {
                    token_string.next();
                    token_vec.push(Token::RedirectAppend);
                } else {
                    token_vec.push(Token::RedirectOut);
                }
            }
            '<' => {
                if unquoted && !string_buffer.is_empty() {
                    unquoted = false;
                    word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
                }
                if !word_buffer.is_empty() {
                    unquoted = false;
                    token_vec.push(Token::Word(std::mem::take(&mut word_buffer)));
                }

                token_vec.push(Token::RedirectIn);
            }
            ';' | '(' | ')' | '`' => {
                return Err(TokenizeError::InvalidOperator(character.to_string()));
            }
            _ => {
                unquoted = true;
                string_buffer.push(character);
            }
        }
    }

    if unquoted && !string_buffer.is_empty() {
        word_buffer.push(WordPart::Word(std::mem::take(&mut string_buffer)));
    }

    if !word_buffer.is_empty() {
        token_vec.push(Token::Word(std::mem::take(&mut word_buffer)));
    }

    Ok(token_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_words_and_symbols() {
        let input = "ls -l /tmp/file.txt";
        let expected = vec![
            Token::Word(vec![WordPart::Word("ls".to_string())]),
            Token::Word(vec![WordPart::Word("-l".to_string())]),
            Token::Word(vec![WordPart::Word("/tmp/file.txt".to_string())]),
        ];
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    fn test_adjacent_quotes() {
        let input = "echo \"hello\"world";
        let expected = vec![
            Token::Word(vec![WordPart::Word("echo".to_string())]),
            Token::Word(vec![
                WordPart::DoubleQuoted("hello".to_string()),
                WordPart::Word("world".to_string()),
            ]),
        ];
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    fn test_operators_without_spaces() {
        let input = "cat<file.txt|grep foo>out.txt";
        let expected = vec![
            Token::Word(vec![WordPart::Word("cat".to_string())]),
            Token::RedirectIn,
            Token::Word(vec![WordPart::Word("file.txt".to_string())]),
            Token::Pipe,
            Token::Word(vec![WordPart::Word("grep".to_string())]),
            Token::Word(vec![WordPart::Word("foo".to_string())]),
            Token::RedirectOut,
            Token::Word(vec![WordPart::Word("out.txt".to_string())]),
        ];
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    fn test_logical_chaining_operators() {
        let input = "make || echo \"failed\" && exit";
        let expected = vec![
            Token::Word(vec![WordPart::Word("make".to_string())]),
            Token::Or,
            Token::Word(vec![WordPart::Word("echo".to_string())]),
            Token::Word(vec![WordPart::DoubleQuoted("failed".to_string())]),
            Token::And,
            Token::Word(vec![WordPart::Word("exit".to_string())]),
        ];
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    fn test_unclosed_quote_error() {
        let input = "echo \"hello";
        assert_eq!(tokenize(input), Err(TokenizeError::UnclosedQuote));

        let input_single = "echo 'hello";
        assert_eq!(tokenize(input_single), Err(TokenizeError::UnclosedQuote));
    }

    #[test]
    fn test_invalid_operator() {
        let input = "sleep 5 &";
        assert_eq!(
            tokenize(input),
            Err(TokenizeError::InvalidOperator("&".to_string()))
        );
    }
}
