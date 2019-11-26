use std::sync::mpsc::*;

pub fn main() {
    let code = "(Atan2 
                    (SiN ( - X x))
                    ( / x y ))";

    // Can use crossbeam-utils here as well for scoped thread
    rayon::scope(|s| {
        let (sender, receiver) = channel();
        s.spawn(|_| {
            Lexer::begin_lexing(&code, sender);
        });
        while let Ok(token) = receiver.recv() {
            println!("Token received from channel: {:?}", token);
        }
    });
}

// Tokens also contain the line number they occurred on
// So a parser could report errors with line numbers
#[derive(Debug)]
pub enum Token<'a> {
    OpenParen(usize),
    CloseParen(usize),
    Operation(&'a str, usize),
    Constant(&'a str, usize),
}

// Function pointer definition must be wrapped in a struct to be recursive
struct StateFunction(fn(&mut Lexer) -> Option<StateFunction>);

pub struct Lexer<'a> {
    input: &'a str,
    start: usize,
    pos: usize,
    width: usize,
    token_sender: Sender<Token<'a>>,
    current_line: usize,
}

impl<'a> Lexer<'a> {
    pub fn begin_lexing(s: &'a str, sender: Sender<Token<'a>>) {
        let mut lexer = Lexer::<'a> {
            input: s,
            start: 0,
            pos: 0,
            width: 0,
            token_sender: sender,
            current_line: 0,
        };
        lexer.run();
    }

    fn run(&mut self) {
        let mut state = Some(StateFunction(Lexer::determine_token));
        while let Some(next_state) = state {
            state = next_state.0(self)
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            self.width = 0;
            None
        } else {
            self.width = 1; // Assuming one always for now
            let c = self.input[self.pos..].chars().next().unwrap();
            if Lexer::is_linebreak(c) {
                self.current_line += 1;
            }
            self.pos += self.width;
            Some(c)
        }
    }

    fn backup(&mut self) {
        self.pos -= 1;
    }

    fn ignore(&mut self) {
        self.start = self.pos;
    }

    fn emit(&mut self, token: Token<'a>) {
        println!("Sending token on channel: {:?}", token);
        self.token_sender
            .send(token)
            .expect("Unable to send token on channel");
        self.start = self.pos;
    }

    fn accept(&mut self, valid: &str) -> bool {
        match self.next() {
            Some(n) if valid.contains(n) => true,
            _ => {
                self.backup();
                false
            }
        }
    }

    fn accept_run(&mut self, valid: &str) {
        while let Some(n) = self.next() {
            if !valid.contains(n) {
                break;
            }
        }
        self.backup();
    }

    fn lex_operation(l: &mut Lexer) -> Option<StateFunction> {
        l.accept_run("+-/*abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        l.emit(Token::Operation(&l.input[l.start..l.pos], l.current_line));
        Some(StateFunction(Lexer::determine_token))
    }

    fn lex_number(l: &mut Lexer) -> Option<StateFunction> {
        l.accept("-");
        let digits = "0123456789";
        l.accept_run(digits);
        if l.accept(".") {
            l.accept_run(digits);
        }
        if &l.input[l.start..l.pos] == "-" {
            // special case - could indicate start of number, or subtract operation
            l.emit(Token::Operation(&l.input[l.start..l.pos], l.current_line));
        } else {
            l.emit(Token::Constant(&l.input[l.start..l.pos], l.current_line));
        }
        Some(StateFunction(Lexer::determine_token))
    }

    fn determine_token(l: &mut Lexer) -> Option<StateFunction> {
        while let Some(c) = l.next() {
            if Lexer::is_white_space(c) {
                l.ignore();
            } else if c == '(' {
                l.emit(Token::OpenParen(l.current_line));
            } else if c == ')' {
                l.emit(Token::CloseParen(l.current_line));
            } else if Lexer::is_start_of_number(c) {
                return Some(StateFunction(Lexer::lex_number));
            } else {
                return Some(StateFunction(Lexer::lex_operation));
            }
        }
        None
    }

    fn is_start_of_number(c: char) -> bool {
        (c >= '0' && c <= '9') || c == '-' || c == '.'
    }

    fn is_white_space(c: char) -> bool {
        c == ' ' || c == '\n' || c == '\t' || c == '\r'
    }

    fn is_linebreak(c: char) -> bool {
        c == '\n'
    }
}
