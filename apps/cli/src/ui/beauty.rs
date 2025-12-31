use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::stdout;

pub fn print_header() {
    let mut stdout = stdout();
    stdout.execute(SetForegroundColor(Color::Cyan)).unwrap();
    println!("   ___             _         _   _ _           _   ");
    println!("  / __|_ ___ _ ___| |___ _ _| |_(_) |_ ___ _ _| |_ ");
    println!(" | _| ' \\ V / _` | '_| '_|  _| |  _/ -_) '_|  _|");
    println!(" |___|_||_\\_/\\__,_|_| |_|  \\__|_|\\__\\___|_|  \\__|");
    println!("      EnvArchitect - Intelligent Setup Tool");
    stdout.execute(ResetColor).unwrap();
    println!();
}
