extern crate colorful;
use cli_table::{Cell, Style, Table, format::Justify, print_stdout};
use colorful::core::color_string::CString;
use colorful::{Color, Colorful};
use filesize;
use human_bytes::human_bytes;
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir;
use walkdir::WalkDir;

fn main() {
    let directory = env::current_dir().expect("Failed to get current directory");
    let mut directory = String::from(directory.to_str().unwrap());

    // directory.push_str("/src");

    let mut directory_contents = get_directory_contents(&directory);
    sort_directory_by_extension(&mut directory_contents);
    let directory_position = directory_contents.iter().position(|x| x.file_is_directory);
    if !directory_position.is_none() {
        let directory_position = directory_position.unwrap();
        sort_directory_alphabetically(&mut directory_contents[directory_position..]);
    }

    let mut sum_of_filesize = 0;
    let table = directory_contents
        .iter()
        .enumerate()
        .map({
            let mut sum_of_filesize = &mut sum_of_filesize;
            move |(index, x1)| {
                *sum_of_filesize += x1.file_size;
                return vec![
                    (index + 1).cell().justify(Justify::Center),
                    x1.file_name.clone().cell().justify(Justify::Left),
                    (if x1.file_extension.is_none() {
                        String::from("-")
                    } else {
                        x1.file_extension.clone().unwrap()
                    })
                    .cell()
                    .justify(Justify::Center),
                    (if x1.file_is_directory { "Dir" } else { "File" })
                        .cell()
                        .justify(Justify::Center),
                    format_filesize(x1.file_size)
                        .cell()
                        .justify(Justify::Center),
                    format_filesize(x1.file_disk_size)
                        .cell()
                        .justify(Justify::Center),
                ];
            }
        })
        .collect::<Vec<_>>()
        .table()
        .title(vec![
            "No.".cell().bold(true).justify(Justify::Center),
            "File Name".cell().bold(true).justify(Justify::Center),
            "File Extension".cell().bold(true).justify(Justify::Center),
            "File Type".cell().bold(true).justify(Justify::Center),
            "File Size".cell().bold(true).justify(Justify::Center),
            "File Disk Size".cell().bold(true).justify(Justify::Center),
        ])
        .bold(true);

    print_stdout(table).expect("Failed to print table");
    println!("\nTotal Size: {}", format_filesize(sum_of_filesize));
}

fn format_filesize(filesize: u64) -> CString {
    let upper_threshold: u64 = 50 * 1000 * 1000; // 50 gigabytes
    let mut color: Option<Color> = None; // Default color
    if filesize >= upper_threshold {
        color = Option::from(Color::LightRed);
    } else if filesize > 524288000 && filesize <= 1073741824 {
        color = Option::from(Color::Yellow1)
    } else if filesize <= 524288000 {
        color = Option::from(Color::PaleGreen1a);
    }

    if !color.is_none() {
        human_bytes(filesize as f64).color(color.unwrap())
    } else {
        CString::new(human_bytes(filesize as f64))
    }
}

fn sort_directory_by_extension(contents: &mut Vec<FileInfo>) {
    contents.sort_by(|x, x1| {
        if x.file_extension.is_none() && !x1.file_extension.is_none() {
            return Ordering::Equal;
        };
        return Ordering::Less;
    })
}

/// Case-insensitive sorting of directory names
fn sort_directory_alphabetically(directories: &mut [FileInfo]) {
    directories.sort_by(|x, x1| x.file_name.to_lowercase().cmp(&x1.file_name.to_lowercase()));
}

fn get_directory_contents(path: &String) -> Vec<FileInfo> {
    let contents = fs::read_dir(path).expect("Failed to read directory");
    let mut vector: Vec<FileInfo> = Vec::new();
    for entry in contents {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let file_name = String::from(entry.file_name().to_str().unwrap());
        // let file_size = entry.metadata().unwrap().size();

        // Handle file size calculation differently based on the file type
        let file_size: u64 = match file_type.is_dir() {
            true => {
                let mut file_size = 0;
                for entry in WalkDir::new(entry.path()) {
                    let entry = entry.unwrap();
                    file_size = entry.metadata().unwrap().size();
                }
                file_size
            }
            false => entry.metadata().unwrap().size(),
        };

        // Handle file disk size calculation differently based on the file type
        let file_disk_size: u64 = match file_type.is_dir() {
            true => {
                let mut file_disk_size = 0;
                for entry in WalkDir::new(entry.path()) {
                    let entry = entry.unwrap();
                    file_disk_size = filesize::file_real_size(entry.path()).unwrap_or(0);
                }
                file_disk_size
            }
            false => filesize::file_real_size(entry.path()).unwrap_or(0),
        };

        let file_extension = if file_type.is_file() {
            Some(String::from(match Path::new(&file_name).extension() {
                Some(ext) => ext.to_str().unwrap(),
                None => &file_name,
            }))
        } else {
            None
        };
        let file_info = FileInfo {
            file_is_directory: file_type.is_dir(),
            file_name: file_name.clone(),
            file_extension: if file_extension.is_none() {
                None
            } else {
                Some(String::from(file_extension.unwrap()))
            },
            file_size,
            file_disk_size,
        };

        vector.push(file_info);
    }

    vector
}

struct FileInfo {
    file_name: String,
    file_extension: Option<String>,
    file_size: u64,
    file_disk_size: u64,
    file_is_directory: bool,
}
