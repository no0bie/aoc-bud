use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use dotenv::dotenv;
use env::var;
use native_tls::{TlsConnector, TlsStream};

const HOST: &str = "adventofcode.com";
const PORT: u16 = 443;
const INPUT_FOLDER: &str = "aoc_inputs";

#[cfg(feature = "test")]
use regex::Regex;

#[cfg(feature = "time")]
use time::{macros::offset, OffsetDateTime};

struct AocInput {
    path: String,
}

impl AocInput {
    fn new(day: u8, year: i32) -> Result<Self, String> {
        if !Path::new(INPUT_FOLDER).exists() {
            fs::create_dir(INPUT_FOLDER).map_err(|e| {
                format!("Failed to create directory 'aoc_inputs'.\nFs create_dir error: {e}")
            })?;
        }

        return Ok(AocInput {
            path: format!("./{INPUT_FOLDER}/{day}_{year}.input"),
        });
    }

    fn read(&self) -> Option<String> {
        if Path::new(&self.path).exists() {
            return Some(
                fs::read_to_string(&self.path).expect("The file exits but it can't be read"),
            );
        }
        None
    }

    fn write(&self, contents: &str) -> Result<(), String> {
        fs::write(&self.path, contents)
            .map_err(|e| format!("Failed to write content problem input.\nFs write error: {e}"))
    }
}

struct Client {
    session: String,
    stream: TlsStream<TcpStream>,
    path: String,
    input: AocInput,
}

impl Client {
    fn new(day: u8, year: i32) -> Result<Self, String> {
        dotenv().ok();
        let cookie = var("AOC_SESSION").expect("AOC_SESSION must be set in the .env file");

        let tcp = TcpStream::connect(format!("{HOST}:{PORT}"))
            .map_err(|e| format!("Unable to connect to adventofcode, maybe your internet is down?\nTcpStream Error: {}", e))?;

        let connector = TlsConnector::new().expect("Unable to create a TlsConnector");

        let stream = connector.connect(HOST, tcp)
            .map_err(|e| format!("Unable to connect to adventofcode, maybe your internet is down?\nTlsStream Error: {}", e))?;

        Ok(Self {
            session: format!("session={}", cookie),
            stream,
            path: format!("/{year}/day/{day}"),
            input: AocInput::new(day, year)?,
        })
    }

    fn get(&mut self, path: &str) -> Result<String, String> {
        let get_request = format!(
            "GET {0}{path} HTTP/1.1\r\nHost: {HOST}\r\nUser-Aget: AocBud-RustHttp\r\nAccept: */*\r\nCookie: {1}\r\nConnection: close\r\n\r\n",
            self.path,
            self.session
        );

        self.stream.write_all(get_request.as_bytes())
            .map_err(|e| format!("Couldn't perform the GET request for the endpoint {path}.\nTlsStream write_all error: {e}"))?;

        let mut buf: Vec<u8> = Vec::new();

        self.stream.read_to_end(&mut buf)
            .map_err(|e| format!("Couldn't read the response for the endpoint {path}.\nTlsStream read_to_end error {e}"))?;

        Ok(String::from_utf8_lossy(&buf)
            .split_once("\r\n\r\n")
            .expect("Response has no body")
            .1
            .to_string())
    }

    fn get_input(&mut self) -> Result<String, String> {
        if let Some(input) = self.input.read() {
            return Ok(input);
        }
        let contents = &mut self.get("/input")?;
        self.input.write(contents)?;
        Ok(contents.to_string())
    }

    #[cfg(feature = "test")]
    fn get_test_input(&mut self) -> Result<String, String> {
        let re = Regex::new(r"(?s)<pre><code>(.*)</code></pre>").unwrap();
        let html = self.get("").unwrap();
        if let Some(captures) = re.captures(&html) {
            if let Some(test_input) = captures.get(1) {
                return Ok(test_input.as_str().to_string());
            }
        }

        Err(String::from("Couldn't find test input"))
    }
}

pub struct AoC {
    client: Client,
}

impl AoC {
    pub fn new(day: u8, year: i32) -> Self {
        match Client::new(day, year) {
            Ok(client) => AoC { client },
            Err(e) => panic!("{e}"),
        }
    }

    pub fn input(&mut self) -> String {
        match self.client.get_input() {
            Ok(contents) => contents.to_string(),
            Err(e) => panic!("{e}"),
        }
    }

    #[cfg(feature = "test")]
    pub fn test_input(&mut self) -> String {
        match self.client.get_test_input() {
            Ok(contents) => contents,
            Err(e) => panic!("{e}"),
        }
    }

    #[cfg(feature = "time")]
    pub fn today() -> Self {
        let date = OffsetDateTime::now_utc().to_offset(offset!(-5));

        match Client::new(date.day(), date.year()) {
            Ok(client) => AoC { client },
            Err(e) => panic!("{e}"),
        }
    }

    pub fn solve(&self) -> Result<(), String> {
        Ok(())
    }
}
