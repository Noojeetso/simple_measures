use super::fileio::{get_filename, read_csv_file, recreate_dir_all};
use super::quartiles::Quartiles;
use crate::errors::{GraphError, GraphErrorRepr};

use anyhow::{Context, Result};
use fs_err as fs;

use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

const TIME_RESULTS_CSV: &str = "total_time.csv";

pub fn prepare_data(
    data_path: &Path,
    preprocessed_data_path: &Path,
    csv_path: &Path,
) -> Result<()> {
    let pack_data_dir = fs::read_dir(data_path)?;
    recreate_dir_all(preprocessed_data_path)?;
    recreate_dir_all(csv_path)?;

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
                    }.into());
                }
            },
            None => {
                eprintln!(
                    "{}: Путь должен быть задан в utf-8 формате",
                    pack_data_dir_entry_path.display()
                );
                return Err(GraphError {
                    repr: GraphErrorRepr::DataPreprocessingError,
                }.into());
            }
        };

        let size_paths = fs::read_dir(&pack_data_dir_entry_path)?;

        let mut int_lines: Vec<Vec<i32>> = Vec::new();

        for size_path_result in size_paths {
            let size_path = size_path_result?.path();
            let filename = size_path.file_name().unwrap();
            let basename = filename.to_str().unwrap().split(".").next().unwrap();

            let file = fs::File::open(&size_path)?;

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

            let algorithm_full_stat_path = preprocessed_data_path
                .join(PathBuf::from_str(format!("{}.txt", algo_name).as_str())?);

            let mut file = fs::File::create(&algorithm_full_stat_path)?;

            file.write_all(full_stats.as_bytes()).unwrap();
        }

        {
            let algorithm_simple_stat_path =
                csv_path.join(PathBuf::from_str(format!("{}.csv", algo_name).as_str())?);

            let simple_stats = int_lines
                .iter()
                .map(|x: &Vec<i32>| format!("{} {}", x[0].to_string(), x[1].to_string()))
                .collect::<Vec<String>>()
                .join("\n");

            let mut file = fs::File::create(&algorithm_simple_stat_path)?;

            file.write_all(simple_stats.as_bytes()).unwrap();
        }
    }

    Ok(())
}

pub fn create_time_total_csv<GenArgT>(
    time_total_dir: &Path,
    csv_path: &Path,
    sizes: &[GenArgT],
    threshold: &Duration,
) -> Result<()>
where
    GenArgT: std::fmt::Display,
{
    use std::cmp::Ordering;
    fn separate_digits_by_groups(row_value_str: &str) -> String {
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

    let mut file_names_with_peak_time: Vec<(String, (usize, usize))> = Vec::new();
    let pack_csv_dir = fs::read_dir(csv_path)?;
    for pack_data_dir_entry in pack_csv_dir {
        let pack_csv_dir_entry_path = pack_data_dir_entry?.path();
        if let Some(ext) = pack_csv_dir_entry_path.extension() {
            if ext == "json" {
                continue;
            }
        }

        let algorithm_name: &str = get_filename(&pack_csv_dir_entry_path)?;

        let file = fs::File::open(&pack_csv_dir_entry_path)?;
        let reader = BufReader::new(file);
        let peak_time_measure = reader
            .lines()
            .scan((), |_, x| x.ok())
            .map(|x| {let mut split = x.split(' ');
                              (split.next().unwrap().parse::<i32>().unwrap(),
                               split.next().unwrap().parse::<i32>().unwrap())})
            .max_by(|a,b|
                match a.0.cmp(&b.0) {
                    Ordering::Equal => a.1.cmp(&b.1),
                    other => other,
                });

        if !peak_time_measure.is_some_and(|x| x.0 >= 0 && x.1 >= 0) {
            eprintln!(
                "{}: Измеренное время и размер данных не могут быть меньше нуля",
                pack_csv_dir_entry_path.display(),
            );
            return Err(GraphError {
                repr: GraphErrorRepr::ParseError,
            }.into());
        }

        let peak_time_measure = peak_time_measure.unwrap();
        let peak_time_measure = (peak_time_measure.0 as usize, peak_time_measure.1 as usize);
        file_names_with_peak_time.push((algorithm_name.to_owned(), peak_time_measure));

        // eprintln!(
        //     "[create_time_total_csv]: algorithm(or dir)_name: {}",
        //     algorithm_name
        // );
    }
    file_names_with_peak_time.sort_by(|a, b|
        match b.1.0.cmp(&a.1.0) {
            Ordering::Equal => a.1.1.cmp(&b.1.1),
            other => other,
        });

    let mut merged_rows: Vec<String> = Vec::new();
    let mut header = String::from("size");
    for entry in file_names_with_peak_time.iter() {
        let file_path = PathBuf::from(entry.0.clone());
        header.push_str(format!(",{}", file_path.file_stem().unwrap().to_str().unwrap()).as_str());
    }
    merged_rows.push(header);
    for size in sizes.iter() {
        let formatted_value = separate_digits_by_groups(size.to_string().as_str());
        let mut formatted_row = String::new();
        formatted_row.push_str(&formatted_value);
        merged_rows.push(formatted_row);
    }

    for entry in file_names_with_peak_time.iter() {
        let file_path = csv_path.join(PathBuf::from_str(&entry.0)?);
        // eprintln!("[create_time_total_csv]: filename: {}", file_path);

        let (_header, rows) = read_csv_file(&file_path, false, 1)
            .with_context(|| format!("Failed to read instrs from {}", file_path.display()))?;
        for i in 0..rows.len() {
            let row = &rows[i];

            let mut formatted_row = String::new();
            for row_value in row {
                formatted_row.push(',');
                let formatted_value = separate_digits_by_groups(row_value.as_str());
                formatted_row.push_str(&formatted_value);
            }
            merged_rows[i+1].push_str(formatted_row.as_str());
        }
        for i in rows.len()..merged_rows.len()-1 {
            merged_rows[i+1].push_str(",>");
            let formatted_value = separate_digits_by_groups(format!("{}", threshold.as_nanos()).as_str());
            merged_rows[i+1].push_str(formatted_value.as_str());
        }
    }

    let merged_string = merged_rows.join("\n");

    let time_total_path = time_total_dir.join(PathBuf::from_str(TIME_RESULTS_CSV)?);
    let mut file = fs::File::create(&time_total_path)?;

    file.write_all(merged_string.as_bytes()).unwrap();

    Ok(())
}
