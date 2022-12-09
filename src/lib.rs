use std::{
    fs,
    env::var, 
    path::Path, 
    process::exit
};

use dotenv::dotenv;
use chrono::Datelike;
use reqwest::blocking;

struct WebScraper{
    url: String,
    session: String,
    client: blocking::Client,
}

impl WebScraper{


    fn get_input(&self) -> String{

        let content: blocking::Response = self.client
        .get(format!("{}/input", self.url))
        .header("cookie", &self.session)
        .send().unwrap();

        if !content.status().is_success(){
            println!("Something went wrong. Status code: {}", content.status());
            exit(1);
        }

        let content: String = content.text().unwrap();
        let content: String = content[0..content.len()-1].to_string();

        if content == "Puzzle inputs differ by user.  Please log in to get your puzzle input."{
            println!("Looks like your session cookie is not working. Failed to authenticate");
            exit(1);
        }

        content
    }

    fn get_level(&self) -> String{
        let content = self.client
        .get(format!("{}", self.url))
        .header("cookie", &self.session)
        .send().unwrap()
        .text().unwrap();

        let document = scraper::Html::parse_document(&content);
        let level_selector = scraper::Selector::parse(r#"input[name="level"]"#).unwrap();

        let level: &str = match document.select(&level_selector).next(){
            Some(element) => element.value().attr("value").unwrap(),
            _ => {
                println!("Something went wrong, have you already completed the puzzle or is your session incorrect? Defaulting to first part");
                "1"
            }

        };

        level.to_string()
    }

    fn test_solution(&self, solution: &String) -> String{

        let content = self.client
        .post(format!("{}/answer", self.url))
        .header("cookie", &self.session)
        .form(&[
            ("level", &self.get_level()),
            ("answer", solution),
        ])
        .send().unwrap()
        .text().unwrap();
        
        let document = scraper::Html::parse_document(&content);
        let answer_selector = scraper::Selector::parse("main>article>p").unwrap();

        let answer = String::from_iter(
            document.select(&answer_selector)
            .next()
            .unwrap()
            .children()
            .map(|child|{

            if child.value().is_text(){
                return child.value().as_text().unwrap() as &str;
            }

            ""
        }).collect::<Vec<&str>>());

        if answer.contains("too recently"){
           return answer;
        }
        else if answer.contains("not the right answer"){
            let mut answer = answer.split(".");
            return format!("{}.{}", answer.nth(0).unwrap(), answer.nth(1).unwrap())
        }

        // Missing correct answer arm

        answer

    }
}

struct AocFile {
    path: String,
}

impl AocFile{
    fn dir_create(){
        if !Path::new("aoc_inputs").exists() {
            match fs::create_dir("aoc_inputs"){
                Ok(val) => val,
                Err(err) => println!("Failed to create dir, maybe you don't have permission to write?\nError : {}", err)
            };
        }
    }

    fn exists(&self) -> bool{
        Self::dir_create();
        Path::new(&self.path).exists()
    }

    fn read(&self) -> String{
        fs::read_to_string(&self.path)
        .expect("Suddenly the file disappeared")
    }

    fn write(&self, content: &String){
        match fs::write(&self.path, content){
            Ok(val) => val,
            Err(err) => println!("Failed to write content, maybe you don't have permission to write?\nError : {}", err)
        };
    }
}

fn _setup_scrapper(day: u8, year: u16) -> WebScraper{
    WebScraper {
        url : format!("https://adventofcode.com/{year}/day/{day}"),
        session : _get_session(),
        client: blocking::Client::new(),
    }
}

fn _setup_file(day: &u8, year: &u16) -> AocFile{
    AocFile {
        path: format!("./aoc_inputs/day{day}_year{year}"),
    }
}

fn _get_session() -> String{
    dotenv().ok();
    let cookie: String = var("AOC_SESSION").expect("AOC_SESSION cookie value must be set.");
    format!("session={cookie}")
}


fn _get_date() -> (u8, u16) {
    let c_date = chrono::Utc::now().date_naive();
    (c_date.day() as u8, c_date.year() as u16)
}

fn _mitm_new(day: u8, year: u16) -> String{
    let file: AocFile = _setup_file(&day, &year);

    if file.exists(){
        return file.read();
    }

    let web_scrapper: WebScraper = _setup_scrapper(day, year);
    let content: String = web_scrapper.get_input();
    
    file.write(&content);
    content
}

fn _mitm_solve(day: u8, year: u16, solution: &String) -> String{
    let web_scraper: WebScraper = _setup_scrapper(day, year);

    web_scraper.test_solution(solution)
}

/// 
///  Requests puzzle from todays date
///     
///     Returns:
/// 
///         puzzle: String -> Unsplitted puzzle input 
///
pub fn new() -> String{
    let (day, year): (u8, u16) = _get_date();
    _mitm_new(day, year)
}

/// 
///  Requests puzzle from specified date
/// 
/// 
///     Parameters:
/// 
///         day: u8   -> Day of the puzzle
/// 
///         year: u16 -> Year of the puzzle
///     
///     Returns:
/// 
///         puzzle: String -> Unsplitted puzzle input 
///
pub fn new_custom(day: u8, year: u16) -> String{
    _mitm_new(day, year)
}

/// 
///  Send your solution to advent of code, automatically detects what part you're on
/// 
/// 
///     Parameters:
/// 
///         solution: &String -> Your solution
///     
///     Returns:
/// 
///         server_message: String -> Parsed server message
/// 
pub fn solve(solution: &String) -> String{
    let (day, year): (u8, u16) = _get_date();

    _mitm_solve(day, year, solution)
}

/// 
///  Send your solution to advent of code.
///  Automatically detects what part you're on.
/// 
/// 
///     Parameters:
/// 
///         day: u8   -> Day of the puzzle you're solving
/// 
///         year: u16 -> Year of the puzzle you're solving
/// 
///         solution: &String -> Your solution
///     
///     Returns:
/// 
///         server_message: String -> Parsed server message
///
pub fn solve_custom(day: u8, year: u16, solution: &String) -> String{
    _mitm_solve(day, year, solution)
}