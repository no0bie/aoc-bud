use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use dotenv::dotenv;
use env::var;
use native_tls::{TlsConnector, TlsStream};

pub use regex::Regex;

const HOST: &str = "adventofcode.com";
const PORT: u16 = 443;
const INPUT_FOLDER: &str = "aoc_inputs";

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

    fn new_test(day: u8, year: i32) -> Result<Self, String> {
        if !Path::new(INPUT_FOLDER).exists() {
            fs::create_dir(INPUT_FOLDER).map_err(|e| {
                format!("Failed to create directory 'aoc_inputs'.\nFs create_dir error: {e}")
            })?;
        }

        return Ok(AocInput {
            path: format!("./{INPUT_FOLDER}/test_{day}_{year}.input"),
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
    day: u8,
    year: i32,
    path: String,
    sol_re: Regex,
    input: AocInput,
    test_input: AocInput,
}

impl Client {
    fn new(day: u8, year: i32) -> Result<Self, String> {
        dotenv().ok();
        let cookie = var("AOC_SESSION").expect("AOC_SESSION must be set in the .env file");

        let sol_re = Regex::new(r"(?P<wrong_answer>That's\snot\sthe\sright\sanswer)|(?P<wrong_level>You\sdon't\sseem\sto\sbe\ssolving\sthe\sright\slevel)|(?P<timeout>You\shave\s.*\sleft\sto\swait)").unwrap();

        Ok(Self {
            session: format!("session={}", cookie),
            day,
            year,
            path: format!("/{year}/day/{day}"),
            sol_re,
            input: AocInput::new(day, year)?,
            test_input: AocInput::new_test(day, year)?,
        })
    }

    fn build_client(&self) -> Result<TlsStream<TcpStream>, String> {
        let tcp = TcpStream::connect(format!("{HOST}:{PORT}"))
            .map_err(|e| format!("Unable to connect to adventofcode, maybe your internet is down?\nTcpStream Error: {}", e))?;

        let connector = TlsConnector::new().expect("Unable to create a TlsConnector");

        connector.connect(HOST, tcp)
            .map_err(|e| format!("Unable to connect to adventofcode, maybe your internet is down?\nTlsStream Error: {}", e))
    }

    fn get(&self, path: &str) -> Result<String, String> {
        let get_request = format!(
            "GET {0}{path} HTTP/1.1\r\nHost: {HOST}\r\nUser-Aget: AocBud-RustHttp\r\nAccept: */*\r\nCookie: {1}\r\nConnection: close\r\n\r\n",
            self.path,
            self.session
        );

        let mut stream = self.build_client()?;

        stream.write_all(get_request.as_bytes())
            .map_err(|e| format!("Couldn't perform the GET request for the endpoint {path}.\nTlsStream write_all error: {e}"))?;

        let mut buf: Vec<u8> = Vec::new();

        stream.read_to_end(&mut buf)
            .map_err(|e| format!("Couldn't read the response for the endpoint {path}.\nTlsStream read_to_end error {e}"))?;

        let binding = String::from_utf8_lossy(&buf);
        let (_, resp_body) = binding
            .split_once("\r\n\r\n")
            .expect("Response has no body");

        self.check_exists(resp_body)?;
        self.check_404(resp_body)?;

        Ok(resp_body.to_string())
    }

    fn post(&self, path: &str, body: String) -> Result<String, String> {
        let post_request = format!(
            "POST {0}{path} HTTP/1.1\r\nHost: {HOST}\r\nUser-Aget: AocBud-RustHttp\r\nAccept: */*\r\nCookie: {1}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {2}\r\nConnection: close\r\n\r\n{body}",
            self.path,
            self.session,
            body.len(),
        );

        let mut stream = self.build_client()?;

        stream.write_all(post_request.as_bytes())
            .map_err(|e| format!("Couldn't perform the POST request for the endpoint {path}.\nTlsStream write_all error: {e}"))?;

        let mut buf: Vec<u8> = Vec::new();

        stream.read_to_end(&mut buf)
            .map_err(|e| format!("Couldn't read the response for the endpoint {path}.\nTlsStream read_to_end error {e}"))?;

        let binding = String::from_utf8_lossy(&buf);
        let (_, resp_body) = binding
            .split_once("\r\n\r\n")
            .expect("Response has no body");

        self.check_exists(resp_body)?;
        self.check_404(resp_body)?;

        Ok(resp_body.to_string())
    }

    fn check_exists(&self, contents: &str) -> Result<(), String> {
        if let Some(_) =
            contents.split_once("The calendar countdown is synchronized with the server time;")
        {
            return Err(format!(
                "The puzzle for day {0} of {1} has not been unlocked yet!",
                self.day, self.year
            ));
        }
        Ok(())
    }

    fn check_404(&self, contents: &str) -> Result<(), String> {
        if let Some(_) = contents.split_once("404 Not Found") {
            return Err(format!(
                "The puzzle for day {0} of {1} does not exist!",
                self.day, self.year
            ));
        }

        Ok(())
    }

    fn get_input(&self) -> Result<String, String> {
        if let Some(input) = self.input.read() {
            return Ok(input);
        }
        let contents: &str = &self.get("/input")?;
        self.input.write(contents)?;
        Ok(contents.to_string())
    }

    fn post_solution(&self, level: u8, answer: String) -> Result<(), String> {
        let html = self.post("/answer", format!("level={level}&answer={answer}"))?;
        if let Some(captures) = self.sol_re.captures(&html) {
            if let Some(_) = captures.name("wrong_answer") {
                return Err("Incorrect solution, try again".to_string());
            } else if let Some(_) = captures.name("wrong_level") {
                return Err("Incorrect level, did you already solve it? Or are you trying to access a level that is not unlocked?".to_string());
            } else if let Some(timeout) = captures.name("timeout") {
                return Err(format!(
                    "You gave an answer too recently. {0}.",
                    timeout.as_str()
                ));
            }
        }
        Ok(())
    }

    fn get_test_input(&self) -> Result<String, String> {
        if let Some(input) = self.test_input.read() {
            return Ok(input);
        }
        let re = Regex::new(r"(?s)<pre><code>(.*)</code></pre>").unwrap();
        let html = self.get("")?;
        if let Some(captures) = re.captures(&html) {
            if let Some(test_input) = captures.get(1) {
                let contents = test_input.as_str();
                self.test_input.write(contents)?;
                return Ok(contents.to_string());
            }
        }

        Err(String::from("Couldn't find test input"))
    }
}

pub struct Aoc {
    client: Client,
}

impl Aoc {
    /// Creates a new [`Aoc`].
    ///
    /// # Arguments
    ///
    /// - `day` -> day of aoc you want to solve
    /// - `year` -> year of aoc you want to solve
    ///
    /// # Examples:
    ///
    /// ```
    /// use aoc_bud::Aoc;
    ///
    /// // Create a new handler for the first aoc day of 2023
    /// let aoc = Aoc::new(1, 2023);
    /// ```
    /// # Panics
    ///
    /// Panics if:
    ///  - .env file is not present
    ///  - AOC_SESSION is not present in .env
    ///  - Current working directory is not writteable
    pub fn new(day: u8, year: i32) -> Self {
        match Client::new(day, year) {
            Ok(client) => Self { client },
            Err(e) => panic!("{e}"),
        }
    }

    /// Returns the input of the puzzle of the corresponding day.
    ///
    /// # Examples:
    ///
    /// ```
    /// use aoc_bud::Aoc;
    ///
    /// let aoc: Aoc = Aoc::new(1, 2023);
    /// let input: String = aoc.input();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if:
    ///  - It can't perform a request
    ///  - Current working directory is not writteable
    ///  - The selected date and year don't have a puzzle yet or don't exist
    pub fn input(&self) -> String {
        match self.client.get_input() {
            Ok(contents) => contents.to_string(),
            Err(e) => panic!("{e}"),
        }
    }

    /// Returns the test input of the puzzle of the corresponding day.
    ///
    /// # Examples:
    ///
    /// ```
    /// use aoc_bud::Aoc;
    ///
    /// let aoc: Aoc = Aoc::new(1, 2023);
    /// let test_input: String = aoc.test_input();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if:
    ///  - It can't perform a request
    ///  - It can't find the test input in the webpage
    ///  - Current working directory is not writteable
    ///  - The selected date and year don't have a puzzle yet or don't exist
    pub fn test_input(&self) -> String {
        match self.client.get_test_input() {
            Ok(contents) => contents,
            Err(e) => panic!("{e}"),
        }
    }

    #[cfg(feature = "time")]
    /// Creates a new [`Aoc`] with the current date.
    /// Only useful if the the aoc is ongoing.
    ///
    /// # Examples:
    ///
    /// ```
    /// use aoc_bud::Aoc;
    ///
    /// // Create a new handler for the current day
    /// let aoc = Aoc::today();
    /// ```
    /// # Panics
    ///
    /// Panics if:
    ///  - .env file is not present
    ///  - AOC_SESSION is not present in .env
    ///  - Current working directory is not writteable
    pub fn today() -> Self {
        let date = OffsetDateTime::now_utc().to_offset(offset!(-5));

        match Client::new(date.day(), date.year()) {
            Ok(client) => Self { client },
            Err(e) => panic!("{e}"),
        }
    }

    /// Send the solution for the first puzzle, will return [Ok] if correct.
    ///
    /// # Argument
    ///
    /// - `solution` -> your solution
    ///
    /// # Example:
    ///
    /// ```
    /// use aoc_bud::Aoc;
    ///
    /// let aoc: Aoc = Aoc::new(1, 2023);
    /// let input: String = aoc.input();
    ///
    /// let solution = get_solution(input);
    /// // Unwrap so we can get the result
    /// aoc.solve1(solution).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///  - It can't perform a request
    ///  - The solution is incorrect
    ///  - You sent the solution too quickly
    ///  - You are trying to solve a puzzle already solved
    ///  - The selected date and year don't have a puzzle yet or don't exist
    pub fn solve1<T: ToString>(&self, solution: T) -> Result<(), String> {
        self.client.post_solution(1, solution.to_string())
    }

    /// Send the solution for the second puzzle, will return [Ok] if correct.
    ///
    /// # Argument
    ///
    /// - `solution` -> your solution
    ///
    /// # Example:
    ///
    /// ```
    /// use aoc_bud::Aoc;
    ///
    /// let aoc: Aoc = Aoc::new(1, 2023);
    /// let input: String = aoc.input();
    ///
    /// let solution = get_solution(input);
    /// // Unwrap so we can get the result
    /// aoc.solve2(solution).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///  - It can't perform a request
    ///  - The solution is incorrect
    ///  - You sent the solution too quickly
    ///  - You are trying to solve a puzzle already solved
    ///  - You are trying to solve the 2nd puzzle without having solved the 1st
    ///  - The selected date and year don't have a puzzle yet or don't exist
    pub fn solve2<T: ToString>(&self, solution: T) -> Result<(), String> {
        self.client.post_solution(2, solution.to_string())
    }
}
