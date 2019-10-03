use cookie::{Cookie, CookieJar};
use crypto::aes::{cbc_decryptor, KeySize};
use crypto::blockmodes::NoPadding;
use crypto::buffer::*;
use crypto::hmac::Hmac;
use crypto::pbkdf2::pbkdf2;
use crypto::sha1::Sha1;
use error::RemoteError;
use sqlite;
use sqlite::State;
use std::path::Path;

pub fn load_chromium_cookies(db_path: &Path) -> Result<CookieJar, RemoteError> {
    let pass: &str = "peanuts";
    let salt: &str = "saltysalt";
    let mut key = [0u8; 16];
    let mut mac = Hmac::new(Sha1::new(), pass.as_bytes());
    pbkdf2(&mut mac, salt.as_bytes(), 1, &mut key);
    let iv = [32u8; 16];
    let mut jar = CookieJar::new();
    let connection = sqlite::open(db_path)?;
    let mut statement = connection
        .prepare("SELECT host_key, path, is_secure, name, value, encrypted_value FROM cookies")?;
    while let State::Row = statement.next()? {
        let host = statement.read::<String>(0)?;
        let path = statement.read::<String>(1)?;
        let secure = statement.read::<i64>(2)?;
        let name = statement.read::<String>(3)?;
        let mut value = statement.read::<String>(4)?;
        let encrypted_value = &statement.read::<Vec<u8>>(5)?[3..];
        if value == "" {
            let mut buffer = [0u8; 4096];
            let mut value_buffer = Vec::new();
            let mut read_buffer = RefReadBuffer::new(encrypted_value);
            let mut write_buffer = RefWriteBuffer::new(&mut buffer);
            let mut decryptor = cbc_decryptor(KeySize::KeySize128, &key, &iv, NoPadding);
            loop {
                let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
                value_buffer.extend(
                    write_buffer
                        .take_read_buffer()
                        .take_remaining()
                        .iter()
                        .map(|&i| i),
                );
                match result {
                    BufferResult::BufferUnderflow => break,
                    BufferResult::BufferOverflow => {}
                }
            }
            if value_buffer.is_empty()
                || *value_buffer.last().unwrap() as usize > value_buffer.len()
            {
                return Err(RemoteError::new("decrypt error".to_string()));
            }
            value_buffer.truncate(value_buffer.len() - *value_buffer.last().unwrap() as usize);
            value = String::from_utf8(value_buffer)?;
        }
        println!("{} {} {}", host, name, value);
        jar.add(
            Cookie::build(name, value)
                .domain(host)
                .path(path)
                .secure(secure != 0)
                .finish(),
        );
    }
    Ok(jar)
}

pub fn load_firefox_cookies(db_path: &Path) -> Result<CookieJar, RemoteError> {
    let mut jar = CookieJar::new();
    let connection = sqlite::open(db_path)?;
    let mut statement =
        connection.prepare("SELECT host, path, isSecure, name, value FROM moz_cookies")?;
    while let State::Row = statement.next()? {
        let host = statement.read::<String>(0)?;
        let path = statement.read::<String>(1)?;
        let secure = statement.read::<i64>(2)?;
        let name = statement.read::<String>(3)?;
        let value = statement.read::<String>(4)?;
        jar.add(
            Cookie::build(name, value)
                .domain(host)
                .path(path)
                .secure(secure != 0)
                .finish(),
        );
    }
    Ok(jar)
}

fn get_cookie_by_name<'a>(jar: CookieJar, name: &str, host: &str) -> Option<Cookie<'a>> {
    for cookie in jar.iter() {
        if let Some(domain) = cookie.domain() {
            if domain == host && cookie.name() == name {
                return Some(cookie.clone());
            }
        }
    }
    None
}

pub fn get_chromium_cookie<'a>(db_path: &Path, host: &str, name: &str) -> Option<Cookie<'a>> {
    if let Ok(jar) = load_chromium_cookies(db_path) {
        get_cookie_by_name(jar, name, host)
    } else {
        None
    }
}

pub fn get_firefox_cookie<'a>(db_path: &Path, host: &str, name: &str) -> Option<Cookie<'a>> {
    if let Ok(jar) = load_firefox_cookies(db_path) {
        get_cookie_by_name(jar, name, host)
    } else {
        None
    }
}
