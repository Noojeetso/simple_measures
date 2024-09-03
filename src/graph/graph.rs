use super::config::GraphConfig;
use super::fileio::create_file_from_string;
use super::fileio::recreate_dir_all;
use super::preprocess;
use crate::description::PackMeasuresDescription;

use anyhow::Result;
use fs_err as fs;

use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

const PACKS_DIR: &str = "packs";
const DATA_DIR: &str = "data";
const TEMP_DIR: &str = "graph_temp";
const CSV_DIR: &str = "csv";
const PREPROCESSED_DATA_DIR: &str = "preprocessed_data";
const GRAPH_GPI_FILE: &str = "graph.gpi";
const GRAPH_CONFIG_FILE: &str = "graph.conf";
const PACK_DESCRIPTION_FILE: &str = "description.json";

fn add_plot(
    mut gnuplot_str: String,
    pack_name: &str,
    scale_x: f64,
    scale_y: f64,
    filename: &str,
    description: &str,
) -> String {
    let data_path = format!(
        "\"{}/{}/{}/{}/{}.txt\"",
        PACKS_DIR, pack_name, TEMP_DIR, PREPROCESSED_DATA_DIR, filename
    );
    let new_plot_str = format!(
        "\t{} using ($1*({})):($4*({})) with linespoints title \"{}\", \\\n",
        data_path, scale_x, scale_y, description
    );
    gnuplot_str.push_str(&new_plot_str);
    gnuplot_str
}

fn generate_pack_gpi<GenArgT>(
    pack_description: &PackMeasuresDescription<GenArgT>,
    config: &GraphConfig,
) -> Result<()>
where
    GenArgT: std::fmt::Display,
{
    let gnuplot_config = std::include_str!("gnuplot_base_config.gpi");
    let mut gnuplot_str = gnuplot_config.to_string();
    gnuplot_str.push('\n');
    gnuplot_str.push_str(format!("set term {}\n", config.output_type).as_str());
    gnuplot_str.push_str(
        format!(
            "set output \"{}/{}/{}_graph.{}\"\n",
            PACKS_DIR, pack_description.filename, pack_description.filename, config.output_type
        )
        .as_str(),
    );
    gnuplot_str.push('\n');
    gnuplot_str.push('\n');
    gnuplot_str.push_str("#Labels\n");
    gnuplot_str.push('\n');
    gnuplot_str.push_str(format!("set xlabel \"{}\"\n", pack_description.x_label).as_str());
    gnuplot_str.push('\n');
    gnuplot_str.push_str(format!("set ylabel \"{}\"\n", pack_description.y_label).as_str());
    let mut ranges = String::new();
    ranges.push('\n');
    ranges.push_str(format!("# Ranges\n").as_str());
    if config.x_start < config.x_end {
        ranges.push_str(format!("set xrange [{}:{}]\n", config.x_start, config.x_end).as_str());
    }
    if config.y_start < config.y_end {
        ranges.push_str(format!("set yrange [{}:{}]\n\n", config.y_start, config.y_end).as_str());
    }
    gnuplot_str.push_str(ranges.as_str());
    if config.log_x || config.log_y {
        gnuplot_str.push('\n');
        gnuplot_str.push_str("# Logarithmic scale\n");
    }
    if config.log_x {
        gnuplot_str.push('\n');
        gnuplot_str.push_str("set logscale x 10\n\n"); // default logscale is log_10()
    }
    if config.log_y {
        gnuplot_str.push('\n');
        gnuplot_str.push_str("set logscale y 10\n\n"); // default logscale is log_10()
    }
    gnuplot_str.push_str("plot");
    for target_description in pack_description.target_descriptions.iter() {
        gnuplot_str = add_plot(
            gnuplot_str,
            &pack_description.filename.as_str(),
            config.x_scale,
            config.y_scale,
            &target_description.filename,
            &target_description.description,
        );
    }
    // for measure in pack_config.measures.iter() {
    //     gnuplot_str = add_plot(
    //         gnuplot_str,
    //         &config.pack_name.as_str(),
    //         config,
    //         &measure.name,
    //         &measure.desc,
    //     );
    // }
    create_file_from_string(
        format!(
            "{}/{}/{}/{}",
            PACKS_DIR, pack_description.filename, TEMP_DIR, GRAPH_GPI_FILE
        )
        .as_str(),
        &gnuplot_str,
    )?;
    Ok(())
}

fn create_graph_config<GenArgT>(pack_description: &PackMeasuresDescription<GenArgT>) -> Result<()>
where
    GenArgT: std::fmt::Display,
{
    let graph_config = std::include_str!("graph.conf").to_string();
    create_file_from_string(
        format!(
            "{}/{}/{}",
            PACKS_DIR, pack_description.filename, GRAPH_CONFIG_FILE
        )
        .as_str(),
        &graph_config,
    )?;
    Ok(())
}

fn run_gnuplot(pack_name: &str) -> Result<()> {
    let gnuplot_file = format!(
        "{}/{}/{}/{}",
        PACKS_DIR, pack_name, TEMP_DIR, GRAPH_GPI_FILE
    );
    let mut process_handler = if cfg!(target_os = "windows") {
        Command::new("gnuplot.exe")
            .args([gnuplot_file.as_str(), "-persist"])
            .spawn()?
        // .expect("Не удалось запустить процесс")
    } else {
        Command::new("gnuplot")
            .args([gnuplot_file.as_str(), "-persist"])
            .spawn()?
        // .expect(format!("Не удалось запустить процесс {}", "/usr/bin/gnuplot").as_str())
    };
    _ = process_handler.wait();
    Ok(())
}

fn create_temp(pack_name: &str) -> Result<()> {
    let temp_path =
        PathBuf::from_str(format!("{}/{}/{}", PACKS_DIR, pack_name, TEMP_DIR).as_str())?;
    recreate_dir_all(&temp_path)?;
    Ok(())
}

fn clean_temp(pack_name: &str) -> Result<()> {
    let graph_temp_data_dir =
        PathBuf::from_str(format!("{}/{}/{}/", PACKS_DIR, pack_name, TEMP_DIR).as_str())?;
    if graph_temp_data_dir.is_dir() {
        fs::remove_dir_all(graph_temp_data_dir)?;
    }
    Ok(())
}

pub fn generate_single_graphic<GenArgT>(pack_name: &str) -> Result<()>
where
    GenArgT: std::fmt::Display + serde::de::DeserializeOwned,
{
    let pack_descsription_path = PathBuf::from_str(
        format!("{}/{}/{}", PACKS_DIR, pack_name, PACK_DESCRIPTION_FILE).as_str(),
    )?;
    let pack_description = PackMeasuresDescription::<GenArgT>::read(&pack_descsription_path)?;

    let graph_config_path =
        PathBuf::from_str(format!("{}/{}/{}", PACKS_DIR, pack_name, GRAPH_CONFIG_FILE).as_str())?;
    if !graph_config_path.exists() {
        create_graph_config(&pack_description)?;
    }
    let graph_config = GraphConfig::read(&graph_config_path)?;

    create_temp(pack_name)?;
    let data_path =
        PathBuf::from_str(format!("{}/{}/{}", PACKS_DIR, pack_name, DATA_DIR).as_str())?;
    let preprocessed_data_path = PathBuf::from_str(
        format!(
            "{}/{}/{}/{}",
            PACKS_DIR, pack_name, TEMP_DIR, PREPROCESSED_DATA_DIR
        )
        .as_str(),
    )?;
    let csv_path = PathBuf::from_str(
        format!("{}/{}/{}/{}", PACKS_DIR, pack_name, TEMP_DIR, CSV_DIR).as_str(),
    )?;
    preprocess::prepare_data(&data_path, &preprocessed_data_path, &csv_path)?;
    preprocess::create_time_total_csv(
        &data_path,
        &csv_path,
        &pack_description.sizes,
        &pack_description.threshold,
    )?;
    generate_pack_gpi(&pack_description, &graph_config)?;
    run_gnuplot(pack_name)?;
    if !graph_config.save_temp_files {
        clean_temp(pack_name)?;
    }

    Ok(())
}
