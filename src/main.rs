use ignore::gitignore::Gitignore;
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

static ALLOWED_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "md", "rs", "py", "js", "jsx", "html", "css", "scss", "json", "yaml", "yml",
    "toml",
];

struct FileInfo {
    path: String,
    line_count: usize,
    extension: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let files = list_files(Path::new(path));
    let mut extension_map: HashMap<String, usize> = HashMap::new();
    for file in files.iter() {
        *extension_map.entry(file.extension.clone()).or_insert(0) += file.line_count;
    }
    println!("{:<10} | {:<10}", "Extension", "Line Count");
    println!("--------------------------");
    for (extension, line_count) in extension_map {
        println!("{:<10} | {:<10}", extension, line_count);
    }

    println!("");
    // print top 5 biggest files per extension
    let mut extension_files: HashMap<String, Vec<&FileInfo>> = HashMap::new();
    for file in files.iter() {
        extension_files
            .entry(file.extension.clone())
            .or_insert(Vec::new())
            .push(file);
    }
    for (extension, files) in extension_files {
        let mut sorted_files = files;
        sorted_files.sort_by(|a, b| b.line_count.cmp(&a.line_count));
        println!("Top 5 biggest files for extension: {}", extension);
        println!("{:<10} | {:<10} | {}", "Extension", "Line Count", "Path");
        println!("--------------------------------------------");
        for file in sorted_files.iter().take(5) {
            println!(
                "{:<10} | {:<10} | {}",
                extension, file.line_count, file.path
            );
        }
        println!("");
    }
}

fn list_files(dir: &Path) -> Vec<FileInfo> {
    let mut files = Vec::new();
    let (gitignore, _errors) = Gitignore::new(dir.join(".gitignore"));
    if dir.is_dir() {
        match fs::read_dir(dir) {
            Ok(entries) => {
                let mut sub_files: Vec<Vec<FileInfo>> = entries
                    .par_bridge()
                    .filter_map(|entry| {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if gitignore.matched(&path, path.is_dir()).is_ignore() {
                                return None;
                            }
                            if path.is_dir() {
                                Some(list_files(&path))
                            } else {
                                // only add if its in allowed extensions
                                if let Some(ext) = path.extension() {
                                    if ALLOWED_EXTENSIONS.contains(&ext.to_str().unwrap()) {
                                        let line_count = count_lines(&path).unwrap_or(0);
                                        return Some(vec![FileInfo {
                                            path: path.display().to_string(),
                                            line_count,
                                            extension: ext.to_str().unwrap().to_string(),
                                        }]);
                                    }
                                }
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                for sub_file in sub_files.drain(..) {
                    files.extend(sub_file);
                }
            }
            Err(e) => eprintln!("Error reading directory {}: {}", dir.display(), e),
        }
    }
    files
}

fn count_lines(path: &Path) -> io::Result<usize> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    Ok(reader.lines().count())
}
