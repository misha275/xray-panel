use crate::core::{create_tables};
use consoletools::{CommandOutput, CommandConsole};
mod core;


fn main() {
    let mut conn = rusqlite::Connection::open("database.db").unwrap();
    create_tables(&mut conn).unwrap();

    consoletools::register_output_format(
        "prompt",
        consoletools::TextFormat::custom(consoletools::CustomColor::Blue).bold(true),
    );

    let mut console = consoletools::CommandConsole::new(">>");

    console.add_command("echo", "Печатает переданный текст", |args| {
        if args.is_empty() {
            CommandOutput::Error("Использование: echo <текст>".to_string())
        } else {
            CommandOutput::Info(args.join(" "))
        }
    });


    if let Err(err) = console.run() {
        eprintln!("Ошибка консоли: {}", err);
    }
}
