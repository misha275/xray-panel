use crate::core::{create_tables, create_user};
use consoletools::{CommandOutput};
use std::sync::{Arc, Mutex};
mod core;


fn main() {
    let mut initial_conn = rusqlite::Connection::open("database.db").unwrap();
    create_tables(&mut initial_conn).unwrap();
    let conn = Arc::new(Mutex::new(initial_conn));

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

    let conn_for_adduser = Arc::clone(&conn);
    console.add_command("adduser", "add new user to database", move |args| {
        if args.len() != 2 {
            return CommandOutput::Error("Usage: adduser <login> <password>".to_string());
        }
        let login = &args[0];
        let password = &args[1];
        let mut guard = match conn_for_adduser.lock() {
            Ok(g) => g,
            Err(_) => {
                return CommandOutput::Error("Database mutex lock failed".to_string());
            }
        };
        match create_user(&mut guard, login, password) {
            Ok(()) => CommandOutput::Success(format!("User '{}' created", login)),
            Err(err) => CommandOutput::Error(format!("Failed to add user: {}", err)),
        }
    });

    let conn_for_login = Arc::clone(&conn);
    console.add_command("login", "login into account", move |args| {
        if args.len() != 2 {
            return CommandOutput::Error("Usage: login <login> <password>".to_string());
        }
        let login = &args[0];
        let password = &args[1];
        let mut guard = match conn_for_login.lock() {
            Ok(g) => g,
            Err(_) => {
                return CommandOutput::Error("Database mutex lock failed".to_string());
            }
        };
        match core::login(&mut guard, login, password) {
            Ok(true) => CommandOutput::Success(format!("User '{}' logged in successfully", login)),
            Ok(false) => CommandOutput::Error("Invalid login or password".to_string()),
            Err(err) => CommandOutput::Error(format!("Login error: {}", err)),
        }
    });

    let conn_for_addpaydate = Arc::clone(&conn);
    console.add_command("addpaydate", "add new payment date to database", move |args| {
        if args.len() != 2 {
            return CommandOutput::Error("Usage: addpaydate <user_id> <pay_date>".to_string());
        }
        let user_id = &args[0];
        let pay_date = &args[1];
        let mut guard = match conn_for_addpaydate.lock() {
            Ok(g) => g,
            Err(_) => {
                return CommandOutput::Error("Database mutex lock failed".to_string());
            }
        };
        match core::add_payment_date(&mut guard, user_id, pay_date) {
            Ok(()) => CommandOutput::Success(format!("Payment date '{}' added for user '{}'", pay_date, user_id)),
            Err(err) => CommandOutput::Error(format!("Failed to add payment date: {}", err)),
        }
    });

    if let Err(err) = console.run() {
        eprintln!("Ошибка консоли: {}", err);
    }
}
