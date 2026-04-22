#![allow(dead_code)]

use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use rand_core::{OsRng, RngCore};
use rusqlite::{Connection, Result, params};
use uuid::Uuid;


pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "CREATE TABLE IF NOT EXISTS user_auth (
            uuid TEXT PRIMARY KEY,
            login TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_active BOOLEAN NOT NULL DEFAULT TRUE
        )",
        [],
    )?;
    tx.execute(
        "CREATE TABLE IF NOT EXISTS user_data (
            uuid TEXT PRIMARY KEY,
            days_left INTEGER NOT NULL DEFAULT 0,
            payment_dates TEXT NOT NULL DEFAULT '[]',
            traffic_left INTEGER NOT NULL DEFAULT 0,
            total INTEGER NOT NULL DEFAULT 0,
            up INTEGER NOT NULL DEFAULT 0,
            down INTEGER NOT NULL DEFAULT 0,
            ip_address TEXT NOT NULL DEFAULT '[]',
            last_online DATETIME,
            is_online BOOLEAN NOT NULL DEFAULT FALSE,
            additional_info TEXT,
            FOREIGN KEY (uuid) REFERENCES user_auth(uuid) ON DELETE CASCADE
        )",
        [],
    )?;
    tx.commit()?;
    Ok(())
}


pub fn get_uuid() -> String {
    let uuid = Uuid::new_v4().to_string();
    return uuid;
}
pub fn get_tocken() -> String {
    let mut b = [0u8; 32];
    OsRng.fill_bytes(&mut b);
    let mut token = String::with_capacity(b.len() * 2);

    for byte in b {
        use std::fmt::Write;
        write!(&mut token, "{:02x}", byte).unwrap();
    }

    token
}

pub fn hash_password(p: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(p.as_bytes(), &salt).unwrap().to_string()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn create_user(conn: &mut Connection, login: &str, password: &str) -> Result<()> {
    let uuid = get_uuid();
    let password_hash = hash_password(password);
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO user_auth (uuid, login, password_hash) VALUES (?1, ?2, ?3)",
        params![uuid, login, password_hash],
    )?;
    tx.execute(
        "INSERT INTO user_data (uuid, payment_dates) VALUES (?1, '[]')",
        params![uuid],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn login(conn: &mut Connection, login: &str, password: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT password_hash FROM user_auth WHERE login = ?1")?;
    let mut rows = stmt.query(params![login])?;

    if let Some(row) = rows.next()? {
        let stored_hash: String = row.get(0)?;
        Ok(verify_password(&stored_hash, password))
    } else {
        Ok(false)
    }

}


pub fn add_payment_date(conn: &Connection, login: &str, payment_date: &str) -> Result<()> {
    let raw_dates: String = conn.query_row(
        "SELECT payment_dates FROM user_data WHERE login = ?1",
        params![login],
        |row| row.get(0),
    )?;

    let mut dates: Vec<String> = serde_json::from_str(&raw_dates).unwrap_or_default();
    dates.push(payment_date.to_string());
    let serialized = serde_json::to_string(&dates).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "UPDATE user_data SET payment_dates = ?1 WHERE login = ?2",
        params![serialized, login],
    )?;
    Ok(())
}

