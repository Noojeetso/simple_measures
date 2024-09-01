use csv::ReaderBuilder;
use std::error;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

pub type IOResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ошибка перевода строки в число")
    }
}
impl error::Error for ParseError {}

#[derive(Debug, Clone)]
pub struct NonPositiveError;

impl std::fmt::Display for NonPositiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Число отрицательно")
    }
}

impl error::Error for NonPositiveError {}

use num::Integer;
use serde::ser::StdError;

pub fn get_filename(path: &Path) -> IOResult<&str> {
    let file_name = match path.file_name() {
        Some(os_string) => match os_string.to_str() {
            Some(name) => name,
            None => return Err(ParseError.into()),
        },
        None => return Err(ParseError.into()),
    };
    Ok(file_name)
}

pub fn scan_nonnegative_number<T: Integer + std::str::FromStr>() -> IOResult<T>
where
    <T as std::str::FromStr>::Err: StdError,
{
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line = line.trim().to_string();

    // let measure_time = match i32::from_str_radix(&line, 10) {
    let res = match line.parse::<T>() {
        Ok(number) => number,
        Err(_e) => {
            eprintln!("Ошибка сканирования числа");
            return Err(ParseError.into());
            // return Err(Box::new(e));
        }
    };

    if res < T::zero() {
        return Err(NonPositiveError.into());
    }

    Ok(res)
}

pub fn scan_nonnegative_number_prompt<T: Integer + std::str::FromStr>(prompt: &str) -> IOResult<T>
where
    <T as std::str::FromStr>::Err: StdError,
{
    print!("{}", prompt);
    std::io::stdout()
        .flush()
        .expect("Невозможно сбросить буфер потока вывода");
    scan_nonnegative_number::<T>()
}

pub fn read_line() -> String {
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Ошибка чтения строки из потока ввода");

    line
}

pub fn read_prompt(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout()
        .flush()
        .expect("Невозможно сбросить буфер потока вывода");
    let line = read_line();
    println!("");

    line
}

pub fn create_file_from_string(name: &str, data: &String) -> IOResult<()> {
    let mut file_out = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&name)
        .expect("Не удалось создать файл");

    if let Err(e) = file_out.write_all(data.as_bytes()) {
        eprintln!("Ошибка при записи в файл: {}", e);
    };

    Ok(())
}

pub fn read_csv_file(
    file_path: &str,
    has_header: bool,
    start_column_index: usize,
) -> IOResult<(Vec<String>, Vec<Vec<String>>)> {
    let file = File::open(file_path)?;
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(has_header)
        .from_reader(file);

    let headers = csv_reader
        .headers()?
        .clone()
        .to_owned()
        .iter()
        .map(|str| str.to_string())
        .collect::<Vec<String>>();

    let rows: Result<Vec<Vec<String>>, csv::Error> = csv_reader
        .records()
        .map(|record| {
            record.map(|r| {
                let re = r.as_slice().split(" ").collect::<Vec<&str>>();
                re.iter()
                    .skip(start_column_index)
                    .map(|x| String::from(*x))
                    .collect()
            })
        })
        .collect();
    let rows = rows?;

    Ok((headers, rows))
}

pub fn recreate_dir_all(path: &std::path::Path) -> IOResult<()> {
    if path.exists() {
        if !path.is_dir() {
            std::fs::remove_file(path)?;
        } else {
            std::fs::remove_dir_all(path)?;
        }
    }
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn input_vector() -> IOResult<Vec<i32>> {
    let mut line: String = read_line();
    line = line.trim().to_string();
    let parts = line.split(" ").collect::<Vec<&str>>();
    let elements_amount = parts.len();
    if elements_amount == 0 {
        let err = NonPositiveError;
        return Err(err.into());
    }

    let mut vector: Vec<i32> = Vec::new();

    for &part in parts.iter() {
        let number: i32 = match part.parse::<i32>() {
            Ok(num) => num,
            Err(e) => {
                println!("Ошибка сканирования числа");
                return Err(e.into());
            }
        };
        vector.push(number);
    }

    Ok(vector)
}
