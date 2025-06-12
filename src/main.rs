use std::{collections::HashMap, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the file to combine in child directories
    pub file:       String,
    /// Directory to search for subdirectories in.
    #[arg(short, long)]
    pub dir:        Option<String>,
    /// Don't sort the output by key, just keep the order of insertion.
    #[arg(long, default_value_t = false)]
    pub keep_order: bool,
}

fn main() {
    let Args {
        file,
        dir,
        keep_order,
    } = Args::parse();
    let dir = match dir {
        Some(d) => PathBuf::from(d),
        None => std::env::current_dir().expect("Failed to get current directory"),
    };

    if !dir.is_dir() {
        eprintln!("Error: The specified directory does not exist or is not a directory.");
        std::process::exit(1);
    }

    let mut cols = Vec::new();
    let mut dirs = Vec::new();
    for entry in dir.read_dir().expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        if entry.path().is_dir() && entry.path().join(&file).is_file() {
            cols.push(entry.file_name().to_string_lossy().to_string());
            // Only add directories that contain the specified file
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    cols.sort();
    cols.insert(0, "key".to_string());

    if dirs.is_empty() {
        eprintln!(
            "Error: No subdirectories found containing the file '{}'",
            file
        );
        std::process::exit(1);
    }

    let mut output = HashMap::<String, Vec<isize>>::new();
    let mut insertion_order = Vec::new();

    for i in 0..dirs.len() {
        let subdir = &dirs[i];
        let file_path = subdir.join(&file);

        println!("Processing file: {}", file_path.display());

        let mut content = std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read file '{}'", file_path.display()));

        let lines_count = content.lines().count();
        if lines_count == 1 {
            println!(
                "Warning: File '{}' has only one line, converting to single-column format",
                file_path.display()
            );
            // If the file has only one line, we treat it as a single-column file
            content = content.trim().split('\t').collect::<Vec<_>>().join("\n");
        }

        for (j, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // split the line by tab character
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() == 1 {
                match output.get_mut(&j.to_string()) {
                    Some(values) => {
                        values[i] = parts[0].parse::<isize>().unwrap_or_else(|_| {
                            eprintln!("Error: Invalid number in line '{}'", line);
                            std::process::exit(1);
                        });
                    },
                    None => {
                        output.insert(j.to_string(), vec![0; dirs.len()]);
                        insertion_order.push(j.to_string());
                        output.get_mut(&j.to_string()).unwrap()[i] =
                            parts[0].parse::<isize>().unwrap_or_else(|_| {
                                eprintln!("Error: Invalid number in line '{}'", line);
                                std::process::exit(1);
                            });
                    },
                }
            } else if parts.len() == 2 {
                match output.get_mut(parts[0]) {
                    Some(values) => {
                        values[i] = parts[1].parse::<isize>().unwrap_or_else(|_| {
                            eprintln!("Error: Invalid number in line '{}'", line);
                            std::process::exit(1);
                        });
                    },
                    None => {
                        output.insert(parts[0].to_string(), vec![0; dirs.len()]);
                        insertion_order.push(parts[0].to_string());
                        output.get_mut(parts[0]).unwrap()[i] =
                            parts[1].parse::<isize>().unwrap_or_else(|_| {
                                eprintln!("Error: Invalid number in line '{}'", line);
                                std::process::exit(1);
                            });
                    },
                }
            } else {
                eprintln!("Error: Line has more than two parts: '{}'", line);
                std::process::exit(1);
            }
        }
    }

    // now we output to a file called combined_<file>
    let output_file = std::fs::File::create(dir.join(format!("combined_{}", file)))
        .unwrap_or_else(|_| panic!("Failed to create output file 'combined_{}'", file));
    println!("Writing to combined_{}", file);
    let mut writer = std::io::BufWriter::new(output_file);
    for col in cols {
        write!(writer, "{}\t", col).expect("Failed to write header to output file");
    }
    writeln!(writer).expect("Failed to write header to output file");
    let mut output = output.into_iter().collect::<Vec<_>>();
    if keep_order {
        output.sort_by(|(ka, _), (kb, _)| {
            insertion_order
                .iter()
                .position(|k| k == ka)
                .unwrap_or(usize::MAX)
                .cmp(
                    &insertion_order
                        .iter()
                        .position(|k| k == kb)
                        .unwrap_or(usize::MAX),
                )
        });
    } else if output[0].0.parse::<isize>().is_err() {
        output.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
    } else {
        output.sort_by(|(ka, _), (kb, _)| {
            ka.parse::<isize>()
                .unwrap_or_else(|_| panic!("Failed to parse key '{}'", ka))
                .cmp(
                    &kb.parse::<isize>()
                        .unwrap_or_else(|_| panic!("Failed to parse key '{}'", kb)),
                )
        });
    }
    for (key, values) in output {
        writeln!(
            writer,
            "{}\t{}",
            key,
            values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join("\t")
        )
        .expect("Failed to write to output file");
    }
    writer.flush().expect("Failed to flush writer");

    println!("Completed writing to combined_{}", file);
}
