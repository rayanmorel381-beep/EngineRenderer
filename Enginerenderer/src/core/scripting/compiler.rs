use super::bytecode::{Instruction, Opcode};

pub fn compile_expr(src: &str) -> Result<Vec<Instruction>, String> {
    let tokens = tokenize(src)?;
    let mut pos = 0usize;
    let mut instrs = Vec::new();
    parse_expr(&tokens, &mut pos, &mut instrs)?;
    instrs.push(Instruction::new(Opcode::Halt, 0));
    Ok(instrs)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(i64),
    Float(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Bang,
}

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' => { i += 1; }
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            '=' if chars.get(i + 1) == Some(&'=') => { tokens.push(Token::EqEq); i += 2; }
            '!' if chars.get(i + 1) == Some(&'=') => { tokens.push(Token::BangEq); i += 2; }
            '<' if chars.get(i + 1) == Some(&'=') => { tokens.push(Token::LtEq); i += 2; }
            '>' if chars.get(i + 1) == Some(&'=') => { tokens.push(Token::GtEq); i += 2; }
            '<' => { tokens.push(Token::Lt); i += 1; }
            '>' => { tokens.push(Token::Gt); i += 1; }
            '!' => { tokens.push(Token::Bang); i += 1; }
            '0'..='9' | '.' => {
                let start = i;
                let mut is_float = false;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    if chars[i] == '.' { is_float = true; }
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                if is_float {
                    tokens.push(Token::Float(s.parse::<f64>().map_err(|e| e.to_string())?));
                } else {
                    tokens.push(Token::Number(s.parse::<i64>().map_err(|e| e.to_string())?));
                }
            }
            c => return Err(format!("unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}

fn parse_expr(tokens: &[Token], pos: &mut usize, instrs: &mut Vec<Instruction>) -> Result<(), String> {
    parse_comparison(tokens, pos, instrs)
}

fn parse_comparison(tokens: &[Token], pos: &mut usize, instrs: &mut Vec<Instruction>) -> Result<(), String> {
    parse_additive(tokens, pos, instrs)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::EqEq  => { *pos += 1; parse_additive(tokens, pos, instrs)?; instrs.push(Instruction::new(Opcode::Eq, 0)); }
            Token::BangEq => { *pos += 1; parse_additive(tokens, pos, instrs)?; instrs.push(Instruction::new(Opcode::Ne, 0)); }
            Token::Lt    => { *pos += 1; parse_additive(tokens, pos, instrs)?; instrs.push(Instruction::new(Opcode::Lt, 0)); }
            Token::Gt    => { *pos += 1; parse_additive(tokens, pos, instrs)?; instrs.push(Instruction::new(Opcode::Gt, 0)); }
            Token::LtEq  => { *pos += 1; parse_additive(tokens, pos, instrs)?; instrs.push(Instruction::new(Opcode::Le, 0)); }
            Token::GtEq  => { *pos += 1; parse_additive(tokens, pos, instrs)?; instrs.push(Instruction::new(Opcode::Ge, 0)); }
            _ => break,
        }
    }
    Ok(())
}

fn parse_additive(tokens: &[Token], pos: &mut usize, instrs: &mut Vec<Instruction>) -> Result<(), String> {
    parse_multiplicative(tokens, pos, instrs)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Plus => {
                *pos += 1;
                parse_multiplicative(tokens, pos, instrs)?;
                instrs.push(Instruction::new(Opcode::Add, 0));
            }
            Token::Minus => {
                *pos += 1;
                parse_multiplicative(tokens, pos, instrs)?;
                instrs.push(Instruction::new(Opcode::Sub, 0));
            }
            _ => break,
        }
    }
    Ok(())
}

fn parse_multiplicative(tokens: &[Token], pos: &mut usize, instrs: &mut Vec<Instruction>) -> Result<(), String> {
    parse_primary(tokens, pos, instrs)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Star => {
                *pos += 1;
                parse_primary(tokens, pos, instrs)?;
                instrs.push(Instruction::new(Opcode::Mul, 0));
            }
            Token::Slash => {
                *pos += 1;
                parse_primary(tokens, pos, instrs)?;
                instrs.push(Instruction::new(Opcode::Div, 0));
            }
            _ => break,
        }
    }
    Ok(())
}

fn parse_primary(tokens: &[Token], pos: &mut usize, instrs: &mut Vec<Instruction>) -> Result<(), String> {
    if *pos >= tokens.len() {
        return Err("unexpected end of expression".to_string());
    }
    match &tokens[*pos] {
        Token::Number(n) => {
            instrs.push(Instruction::new(Opcode::Push, *n));
            *pos += 1;
        }
        Token::Float(f) => {
            instrs.push(Instruction::new(Opcode::PushFloat, f.to_bits() as i64));
            *pos += 1;
        }
        Token::Minus => {
            *pos += 1;
            parse_primary(tokens, pos, instrs)?;
            instrs.push(Instruction::new(Opcode::Neg, 0));
        }
        Token::Bang => {
            *pos += 1;
            parse_primary(tokens, pos, instrs)?;
            instrs.push(Instruction::new(Opcode::Not, 0));
        }
        Token::LParen => {
            *pos += 1;
            parse_expr(tokens, pos, instrs)?;
            if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                return Err("expected closing parenthesis".to_string());
            }
            *pos += 1;
        }
        t => return Err(format!("unexpected token: {:?}", t)),
    }
    Ok(())
}
