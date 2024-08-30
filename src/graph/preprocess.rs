use super::fileio::{get_filename, read_csv_file, recreate_dir_all};
use super::quartiles::Quartiles;
use crate::errors::{GraphError, GraphErrorRepr, Result};

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::time::Duration;

const CSV_PATH: &str = "csv";
const DATA_PATH: &str = "data";
const PREPROCESSED_DATA_PATH: &str = "preprocessed_data";
const TIME_RESULTS_CSV: &str = "total_time.csv";

pub fn prepare_data(pack_name: &str) -> Result<()> {
    let pack_data_dir = std::fs::read_dir(format!("{}/{}", DATA_PATH, pack_name))?;

    {
        let preprocessed_data_path_name = format!("{}/{}", PREPROCESSED_DATA_PATH, pack_name);
        recreate_dir_all(Path::new(preprocessed_data_path_name.as_str()))?;
    }

    {
        let preprocessed_csv_path_name = format!("{}/{}", CSV_PATH, pack_name);
        recreate_dir_all(Path::new(preprocessed_csv_path_name.as_str()))?;
    }

    for pack_data_dir_entry in pack_data_dir {
        let pack_data_dir_entry_path = pack_data_dir_entry?.path();
        if !pack_data_dir_entry_path.is_dir() {
            continue;
        }

        let algo_name = match pack_data_dir_entry_path.to_str() {
            Some(path) => match path.split("/").last() {
                Some(name) => name,
                None => {
                    return Err(GraphError {
                        repr: GraphErrorRepr::DataPreprocessingError,
                    }
                    .into());
                }
            },
            None => {
                eprintln!("Путь должен быть задан в utf-8 формате");
                return Err(GraphError {
                    repr: GraphErrorRepr::DataPreprocessingError,
                }
                .into());
            }
        };

        let size_paths = std::fs::read_dir(&pack_data_dir_entry_path).expect(
            format!(
                "Ошибка чтения каталога {}",
                &pack_data_dir_entry_path.display()
            )
            .as_str(),
        );

        let mut int_lines: Vec<Vec<i32>> = Vec::new();

        for size_path_result in size_paths {
            let size_path = size_path_result.expect("Ошибка чтения пути файла").path();
            let filename = size_path.file_name().unwrap();
            let basename = filename.to_str().unwrap().split(".").next().unwrap();

            let file = match File::open(&size_path) {
                Err(e) => panic!("Не удалось открыть файл {}: {}", &size_path.display(), e),
                Ok(file) => file,
            };

            let reader = BufReader::new(file);
            let mut res_vec = reader
                .lines()
                .scan((), |_, x| x.ok())
                .map(|x| x.parse::<i32>().unwrap())
                .collect::<Vec<i32>>();
            res_vec.sort();
            // println!("file: {}", size_path.as_os_str().to_str().unwrap());
            // println!("sorted: {:#?}", res_vec);
            let quart = Quartiles::new(&res_vec);
            let quart_values = quart.values();

            int_lines.push(vec![
                basename.parse::<i32>().unwrap(),
                quart_values[0] as i32,
                quart_values[1] as i32,
                quart_values[2] as i32,
                quart_values[3] as i32,
                quart_values[4] as i32,
            ]);
        }

        int_lines.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

        {
            let full_stats = int_lines
                .iter()
                .map(|x: &Vec<i32>| {
                    x.iter()
                        .map(|y| y.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                })
                .collect::<Vec<String>>()
                .join("\n");

            let algorithm_full_stat_path =
                format!("{}/{}/{}.txt", PREPROCESSED_DATA_PATH, pack_name, algo_name);

            let mut file = File::create(&algorithm_full_stat_path)
                .expect(format!("Не удалось открыть файл {}", &algorithm_full_stat_path).as_str());

            file.write_all(full_stats.as_bytes()).unwrap();
        }

        {
            let algorithm_simple_stat_path =
                format!("{}/{}/{}.csv", CSV_PATH, pack_name, algo_name);

            let simple_stats = int_lines
                .iter()
                .map(|x: &Vec<i32>| format!("{} {}", x[0].to_string(), x[1].to_string()))
                .collect::<Vec<String>>()
                .join("\n");

            let mut file = File::create(&algorithm_simple_stat_path).expect(
                format!("Не удалось открыть файл {}", &algorithm_simple_stat_path).as_str(),
            );

            file.write_all(simple_stats.as_bytes()).unwrap();
        }
    }

    Ok(())
}

pub fn create_time_total_csv<GenArgT>(
    pack_name: &str,
    sizes: &[GenArgT],
    threshold: &Duration,
) -> Result<()>
where
    GenArgT: std::fmt::Display,
{
    fn separate_row_value(row_value_str: &str) -> String {
        static DIGITS_IN_GROUP: usize = 3;
        let mut out_str = String::new();
        let chars = row_value_str.chars().collect::<Vec<char>>();
        let length = chars.len();

        let offset = (DIGITS_IN_GROUP - (length) % DIGITS_IN_GROUP) % DIGITS_IN_GROUP;

        out_str.push(chars[0]);
        for i in 1..length {
            if i < length - 1 && (i + offset) % DIGITS_IN_GROUP == 0 {
                out_str.push(' ');
            }
            out_str.push(chars[i]);
        }

        out_str
    }

    let mut file_names: Vec<String> = Vec::new();
    let pack_data_dir = std::fs::read_dir(format!("{}/{}", DATA_PATH, pack_name))?;
    for pack_data_dir_entry in pack_data_dir {
        let pack_data_dir_entry_path = pack_data_dir_entry?.path();
        match pack_data_dir_entry_path.extension() {
            Some(ext) => {
                if ext == "json" {
                    continue;
                }
            }
            None => (),
        }
        let algorithm_name: &str = get_filename(&pack_data_dir_entry_path)?;
        file_names.push(format!("{}.csv", algorithm_name));
        // eprintln!(
        //     "[create_time_total_csv]: algorithm(or dir)_name: {}",
        //     algorithm_name
        // );
    }

    let mut merged_rows: Vec<String> = Vec::new();
    for size in sizes.iter() {
        let formatted_value = separate_row_value(size.to_string().as_str());
        let mut formatted_row = String::new();
        formatted_row.push_str(&formatted_value);
        merged_rows.push(formatted_row);
    }

    for i in 0..file_names.len() {
        let file_path = format!("{}/{}/{}", CSV_PATH, pack_name, file_names[i]);
        // eprintln!("[create_time_total_csv]: filename: {}", file_path);

        let (_header, rows) = read_csv_file(&file_path, false, 1)
            .expect(format!("Не удалось прочесть файл: {}", file_path).as_str());
        for i in 0..rows.len() {
            let row = &rows[i];

            let mut formatted_row = String::new();
            for row_value in row {
                formatted_row.push(',');
                let formatted_value = separate_row_value(row_value.as_str());
                formatted_row.push_str(&formatted_value);
            }
            merged_rows[i].push_str(formatted_row.as_str());
        }
        for i in rows.len()..merged_rows.len() {
            merged_rows[i].push_str(",>");
            let formatted_value = separate_row_value(format!("{}", threshold.as_nanos()).as_str());
            merged_rows[i].push_str(formatted_value.as_str());
        }
    }

    let merged_string = merged_rows.join("\n");
    let merged_csv_path = format!("{}/{}/{}", CSV_PATH, pack_name, TIME_RESULTS_CSV);
    let mut file = File::create(&merged_csv_path)
        .expect(format!("Не удалось создать новый файл {}", &merged_csv_path).as_str());

    file.write_all(merged_string.as_bytes()).unwrap();

    Ok(())
}
