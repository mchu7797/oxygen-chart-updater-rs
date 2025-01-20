mod config;
mod database;
mod parser;

use std::fs;
use std::io::Result;
use std::path::PathBuf;

use tiberius::{Client, Config};
use tokio;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::config::ConfigValues;
use crate::parser::ChartInfo;

#[tokio::main]
async fn main() {
    let config = match config::get_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Config read failed: {:?}", e);
            return;
        }
    };

    println!("Config read successfully.");

    let mut sql_config = Config::from_ado_string(config.database_url.clone().as_str()).unwrap();
    sql_config.trust_cert();

    let tcp_connection = TcpStream::connect(sql_config.get_addr()).await.unwrap();
    tcp_connection.set_nodelay(true).unwrap();

    let mut sql_connection = Client::connect(sql_config, tcp_connection.compat_write()).await.unwrap();

    let (upload_charts, update_charts, delete_charts) = get_all_chart_infos(&config).unwrap();

    database::upload_charts(&mut sql_connection, &upload_charts).await.unwrap();
    database::update_charts(&mut sql_connection, &update_charts).await.unwrap();
    database::delete_charts(&mut sql_connection, &delete_charts).await.unwrap();

    println!("Updated successfully.");
}

pub fn get_ojn_files_from_dir(directory: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut ojn_files = Vec::new();

    if !directory.exists() || !directory.is_dir() {
        return Ok(ojn_files);
    }
    
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "ojn" {
                    ojn_files.push(path);
                }
            }
        }
    }

    Ok(ojn_files)
}

pub fn get_upload_chart_infos(config: &ConfigValues) -> Result<Vec<ChartInfo>> {
    let paths = get_ojn_files_from_dir(&config.upload_directory)?;
    convert_paths_to_chart_infos(paths)
}

pub fn get_update_chart_infos(config: &ConfigValues) -> Result<Vec<ChartInfo>> {
    let paths = get_ojn_files_from_dir(&config.update_directory)?;
    convert_paths_to_chart_infos(paths)
}

pub fn get_delete_chart_infos(config: &ConfigValues) -> Result<Vec<ChartInfo>> {
    let paths = get_ojn_files_from_dir(&config.delete_directory)?;
    convert_paths_to_chart_infos(paths)
}

pub fn get_all_chart_infos(
    config: &ConfigValues,
) -> Result<(Vec<ChartInfo>, Vec<ChartInfo>, Vec<ChartInfo>)> {
    let insert_chart_infos = get_upload_chart_infos(config)?;
    let update_chart_infos = get_update_chart_infos(config)?;
    let delete_chart_infos = get_delete_chart_infos(config)?;

    Ok((insert_chart_infos, update_chart_infos, delete_chart_infos))
}

fn convert_paths_to_chart_infos(paths: Vec<PathBuf>) -> Result<Vec<ChartInfo>> {
    let mut chart_infos = Vec::with_capacity(paths.len());

    for path in paths {
        let chart_info = parser::parse_chart_info(path.to_str().unwrap()).unwrap();
        chart_infos.push(chart_info);
    }

    Ok(chart_infos)
}
