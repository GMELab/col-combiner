# `col-combiner`

Combine a series of identically named tab-separated files from multiple sub-directories into a single file. The sub-directory names are used as the column names in the tab-separated output file.

Files with one column use the line number (0-indexed) as the value of the "key" column in the output file.

Files with two columns use the first column as the value of the "key" column in the output file.

## Installation

Requires the [Rust programming language](https://www.rust-lang.org/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
cargo install --git https://github.com/GMELab/col-combiner
```

## Usage

```bash 
col-combiner <file> [--dir <dir>] # if dir is not specified, the current directory is used
# For example
col-combiner example.txt --dir example_dir
# now we get a file called in example_dir called combined_example.txt
```
