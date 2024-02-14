use tokio::{
    net::{TcpListener, TcpStream},
    io::{self, AsyncWriteExt},
    time::{sleep, Duration}
};
use log::{info, warn, error};
use std::net::{Shutdown};
use crate::intruder::Intruder;
//use std::time::Duration;
use std::error::Error;

struct TelnetStream<'a> {
    stream: &'a TcpStream,
}

impl<'a> TelnetStream<'a> {
    fn new(stream: &'a TcpStream) -> TelnetStream<'a> {
        TelnetStream { stream }
    }

    async fn write_all(&mut self, buf: &[u8]) {
        match self.stream.try_write(buf) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe => {
                    self.close();
                }
                _ => self.close(),
            },
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.stream.try_read(buf) {
            Ok(n) => Ok(n),
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe => {
                    warn!("Connection closed by peer");
                    self.close();
                    Err(e)
                }
                _ => {
                    self.close();
                    Err(e)
                }
            },
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        //let (rdh, wrh) = self.stream.split();
        /*wrh.pool_flush();
        match self.stream.flush() {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe => {
                    warn!("Connection closed by peer");
                    self.close();
                    Err(e)
                }
                _ => {
                    self.close();
                    Err(e)
                }
            },
        }*/
        Ok(())
    }

    fn close(&mut self) {
        /*match self.stream.into_std().unwrap().shutdown(Shutdown::Both) {
            Ok(_) => {
                info!("Connection closed successfully");
            }
            Err(e) => {
                error!("Encountered {:?} while shutting down the TCP stream", e);
            }
        }*/
    }
}

enum TelnetCommand {
    Echo,
    SuppressGoAhead,
    TerminalType,
    TerminalSpeed,
    CarriageReturn,
    ToggleFlowControl,
    LineMode,
    CarriageReturnLineFeed,
    OutputMarking,
    NegotiateSuppressGoAhead,
    CarriageReturnLineFeedCRLF,
}

impl TelnetCommand {
    fn as_bytes(&self) -> &[u8] {
        match self {
            TelnetCommand::Echo => &[0xff, 0xfb, 0x01],
            TelnetCommand::SuppressGoAhead => &[0xff, 0xfb, 0x03],
            TelnetCommand::TerminalType => &[0xff, 0xfd, 0x18],
            TelnetCommand::TerminalSpeed => &[0xff, 0xfd, 0x1f],
            TelnetCommand::CarriageReturn => &[0x0d],
            TelnetCommand::ToggleFlowControl => &[0xff, 0xfe, 0x20],
            TelnetCommand::LineMode => &[0xff, 0xfe, 0x21],
            TelnetCommand::CarriageReturnLineFeed => &[0xff, 0xfe, 0x22],
            TelnetCommand::OutputMarking => &[0xff, 0xfe, 0x27],
            TelnetCommand::NegotiateSuppressGoAhead => &[0xff, 0xfc, 0x05],
            TelnetCommand::CarriageReturnLineFeedCRLF => &[0x0d, 0x0a],
        }
    }
}


fn default_banner() -> String {
    return "

#############################################################################
# UNAUTHORIZED ACCESS TO THIS DEVICE IS PROHIBITED You must have explicit,  #
# authorized permission to access or configure this device.                 #
# Unauthorized attempts and actions to access or use this system may result #
# in civil and/or criminal penalties.                                       #
# All activities performed on this device are logged and monitored.         #
#############################################################################

"
    .to_string();
}

fn print_banner(stream: &TcpStream, banner: Option<String>) -> io::Result<()> {
    let mut stream = stream;

    match banner {
        Some(banner) => {
            stream.try_write(banner.as_bytes())?;
        }
        None => {
            stream.try_write(default_banner().as_bytes())?;
        }
    }

    Ok(())
}

async fn get_telnet_username(stream: &TcpStream, intruder: &mut Intruder) {
    let mut telnet_stream = TelnetStream::new(stream);

    telnet_stream.write_all(b"login: ").await;

    let username = read_until_cr(&telnet_stream.stream).await.unwrap();
    intruder.username = username.trim().to_string().clone();
}

async fn read_until_cr(stream: &TcpStream) -> Result<String, Box<dyn Error>>  {
    let mut telnet_stream = TelnetStream::new(stream);
    let mut buffer = Vec::new();

    'outer: loop {
        telnet_stream.flush().unwrap();

        let mut buf = [0; 1024];
        let n = telnet_stream.read(&mut buf).await; // TODO: this errors out: panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }'
        println!("{:?}", n);

        let n = match n {
            Ok(0) => break,
            Ok(n) => {
                n
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                sleep(Duration::new(0, 500000000)).await;
                continue;
            }
            Err(e) => {
                return Err(e.into())
            }
        };
        if n == 0 {
            return Ok(String::from_utf8(buffer).unwrap());
        }

        let s = match std::str::from_utf8(&buf[..n]) {
            Ok(s) => s,
            Err(e) => {
                warn!("Problem reading telnet stream data: {}", e);
                continue;
            }
        };

        for c in s.chars() {
            if c == '\r' || c == '\n' {
                break 'outer;
            }

            if c.is_ascii() {
                buffer.push(c as u8);
            }
        }
    }

    Ok(String::from_utf8(buffer).unwrap())
}

async fn get_telnet_password(stream: &TcpStream, intruder: &mut Intruder) {
    let mut telnet_stream = TelnetStream::new(stream);

    telnet_stream.write_all(TelnetCommand::Echo.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::SuppressGoAhead.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::TerminalType.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::TerminalSpeed.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::CarriageReturn.as_bytes()).await;
    telnet_stream.write_all(b"Password: ").await;
    telnet_stream.write_all(TelnetCommand::ToggleFlowControl.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::LineMode.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::CarriageReturnLineFeed.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::OutputMarking.as_bytes()).await;
    telnet_stream.write_all(TelnetCommand::NegotiateSuppressGoAhead.as_bytes()).await;

    let mut password = read_until_cr(&telnet_stream.stream).await.unwrap();
    telnet_stream.write_all(TelnetCommand::CarriageReturnLineFeedCRLF.as_bytes()).await;

    password = password.trim().to_string();

    intruder.password = password.clone();
}

pub async fn handle_telnet_client(stream: &TcpStream, mut intruder: &mut Intruder) -> io::Result<()> {
    let _ = print_banner(&stream, None);

    let _ = get_telnet_username(&stream, &mut intruder).await;
    let _ = get_telnet_password(&stream, &mut intruder).await;

    sleep(Duration::new(2, 0)).await;

    Ok(())
}