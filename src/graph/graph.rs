// use nix::NixPath;

// use super::config;
use super::config::GraphConfig;
use super::fileio::create_file_from_string;
use super::preprocess;
use crate::description::{PackMeasuresDescription, TargetDescription};
use crate::errors::Result;
// use crate::fileio::recreate_dir_all;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
// use std::str::FromStr;
// use std::time::Duration;
// use std::string::ParseError;
// use std::vec;

// const CSV_PATH: &str = "csv";
const DATA_PATH: &str = "data";
const PREPROCESSED_DATA_PATH: &str = "preprocessed_data";
const GRAPH_CONFIG_FILES_PATH: &str = "graph_config_files";
const GRAPH_CONFIG_APPENDIX: &str = "graph.gpi";
const TIME_RESULTS_CSV: &str = "total_time.csv";

fn add_plot(
    mut gnuplot_str: String,
    pack_name: &str,
    scale_x: f64,
    scale_y: f64,
    filename: &str,
    description: &str,
) -> String {
    let data_path = format!(
        "\"{}/{}/{}.txt\"",
        PREPROCESSED_DATA_PATH, pack_name, filename
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
    let gnuplot_config = fs::read_to_string("measure_configs/gnuplot_config.gpi")?;
    let mut gnuplot_str = gnuplot_config.clone();
    gnuplot_str.push('\n');
    gnuplot_str.push_str("set term pdf\n");
    gnuplot_str.push_str(format!("set output \"{}_graph.pdf\"\n", config.pack_name).as_str());
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
            &config.pack_name.as_str(),
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
        format!("{}_{}", config.pack_name, GRAPH_CONFIG_APPENDIX).as_str(),
        &gnuplot_str,
    )?;
    Ok(())
}

fn run_gnuplot(pack_name: &str) -> Result<()> {
    let gnuplot_files = [format!("{}_{}", pack_name, GRAPH_CONFIG_APPENDIX)];
    for file in gnuplot_files {
        let mut process_handler = if cfg!(target_os = "windows") {
            Command::new("gnuplot.exe")
                .args([file.as_str(), "-persist"])
                .spawn()?
            // .expect("Не удалось запустить процесс")
        } else {
            Command::new("/usr/bin/gnuplot")
                .args([file.as_str(), "-persist"])
                .spawn()?
            // .expect(format!("Не удалось запустить процесс {}", "/usr/bin/gnuplot").as_str())
        };
        _ = process_handler.wait();
    }
    Ok(())
}

fn clean(pack_name: &str) -> Result<()> {
    use std::path::Path;
    let preprocessed_data_path = Path::new(PREPROCESSED_DATA_PATH);
    if preprocessed_data_path.is_dir() {
        std::fs::remove_dir_all(preprocessed_data_path)?;
    }
    let custom_graph_gpi_name = format!("{}_{}", pack_name, GRAPH_CONFIG_APPENDIX);

    let custom_graph_gpi = Path::new(custom_graph_gpi_name.as_str());
    if custom_graph_gpi.is_file() {
        std::fs::remove_file(custom_graph_gpi)?;
    }
    Ok(())
}

pub fn generate_single_graphic<GenArgT>(pack_name: &str, config_path: &Path) -> Result<()>
where
    GenArgT: std::fmt::Display + serde::de::DeserializeOwned,
{
    let pack_name = pack_name;

    let graph_config = GraphConfig::read(config_path)?;
    let pack_descsription_path =
        PathBuf::from_str(format!("{}/{}/description.json", DATA_PATH, pack_name).as_str())?;
    let pack_description = PackMeasuresDescription::<GenArgT>::read(&pack_descsription_path)?;
    {
        let preproc_path = PathBuf::from_str("data_preproc")?;
        if !preproc_path.is_dir() {
            std::fs::create_dir(&preproc_path)?;
        }
    }
    {
        let preproc_path = PathBuf::from_str("csv")?;
        if !preproc_path.is_dir() {
            std::fs::create_dir(&preproc_path)?;
        }
    }
    {
        let preproc_path = PathBuf::from_str("gpi")?;
        if !preproc_path.is_dir() {
            std::fs::create_dir(&preproc_path)?;
        }
    }
    preprocess::prepare_data(pack_name)?;
    preprocess::create_time_total_csv(
        pack_name,
        &pack_description.sizes,
        &pack_description.threshold,
    )?;
    generate_pack_gpi(&pack_description, &graph_config)?;
    run_gnuplot(pack_name)?;
    // clean(pack_name);
    Ok(())
}

// pub fn generate_graphics() -> Result<()> {
//     let configs: Vec<GraphConfig> = config::read_configs(&PathBuf::from_str("data")?)?;
//     for config in configs.iter() {
//         generate_single_graphic(measure_pack)?;
//     }
//     Ok(())
// }
