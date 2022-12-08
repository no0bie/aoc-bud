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
}

impl WebScraper{
    fn get_response(&self) -> String{
        let client: blocking::Client = blocking::Client::new();

        let content: blocking::Response = client
        .get(&self.url)
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
}

fn _setup_scrapper(day: u8, year: u16) -> WebScraper{
    WebScraper {
        url : format!("https://adventofcode.com/{year}/day/{day}/input"),
        session : _get_session(),
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

fn _mitm(day: u8, year: u16) -> String{
    let file: AocFile = _setup_file(&day, &year);

    if file.exists(){
        return file.read();
    }

    let web_scrapper: WebScraper = _setup_scrapper(day, year);
    let content: String = web_scrapper.get_response();
    
    file.write(&content);
    content
}

pub fn new() -> String{
    let (day, year): (u8, u16) = _get_date();
    _mitm(day, year)
}

pub fn new_custom(day: u8, year: u16) -> String{
    _mitm(day, year)
}