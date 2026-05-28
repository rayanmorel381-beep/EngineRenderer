use std::io::{self, IsTerminal};

pub fn terminal_columns() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v >= 40)
        .unwrap_or(100)
}

pub fn print_framed_panel(title: &str, lines: &[String], width: usize) {
    let inner_width = width + 2;
    let title_block = format!(" {} ", title);
    let title_w = display_width(&title_block);
    let dash_total = inner_width.saturating_sub(title_w);
    let left = "─".repeat(dash_total / 2);
    let right = "─".repeat(dash_total - (dash_total / 2));
    eprintln!("╭{}{}{}╮", left, title_block, right);
    for line in lines {
        print_console_row(&fit_to_width(line, width), width);
    }
    eprintln!("╰{}╯", "─".repeat(inner_width));
}

pub fn fit_to_width(text: &str, width: usize) -> String {
    if display_width(text) <= width.saturating_sub(2) {
        return text.to_string();
    }
    let mut out = String::new();
    let mut used = 0usize;
    let limit = width.saturating_sub(5);
    for c in text.chars() {
        let w = char_display_width(c);
        if used + w > limit {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push_str("...");
    out
}

pub fn print_console_block(title: &str, lines: &[String]) {
    let width = lines
        .iter()
        .map(|l| display_width(l))
        .max()
        .unwrap_or(0)
        .max(display_width(title) + 8)
        .max(28);

    let top = "+".to_string() + &"-".repeat(width + 2) + "+";
    let sep = "+".to_string() + &"-".repeat(width + 2) + "+";
    let bottom = "+".to_string() + &"-".repeat(width + 2) + "+";

    eprintln!("{}", top);
    print_console_row(title, width);
    eprintln!("{}", sep);
    for line in lines {
        print_console_row(line, width);
    }
    eprintln!("{}", bottom);
}

pub fn print_console_row(content: &str, width: usize) {
    if io::stderr().is_terminal() {
        eprint!("| {}", content);
        eprint!("\x1B[{}G", width + 4);
        eprintln!("|");
    } else {
        eprintln!("| {} |", pad_display(content, width));
    }
}

pub fn pad_display(text: &str, width: usize) -> String {
    let current = display_width(text);
    if current >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - current))
    }
}

pub fn display_width(text: &str) -> usize {
    text.chars().map(char_display_width).sum()
}

pub fn char_display_width(c: char) -> usize {
    if c.is_ascii() {
        1
    } else if is_zero_width_char(c) {
        0
    } else {
        2
    }
}

pub fn is_zero_width_char(c: char) -> bool {
    matches!(
        c,
        '\u{200D}'
            | '\u{FE0E}'
            | '\u{FE0F}'
            | '\u{FE20}'..='\u{FE2F}'
            | '\u{0300}'..='\u{036F}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{1F3FB}'..='\u{1F3FF}'
    )
}
