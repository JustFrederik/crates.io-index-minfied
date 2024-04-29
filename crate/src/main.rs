use std::collections::HashMap;
use std::fs;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Ver {
    name: String,
    vers: String,
    yanked: bool,
}

fn read_files_in_directory(dir_path: &Path) -> io::Result<Vec<(String, Vec<String>)>> {
    let mut output = vec![];
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(file_name) = path.file_name() {
                if let Some(file_str) = file_name.to_str() {
                    if file_str.starts_with(".") {
                        continue;
                    }
                }
            }

            if path.is_file() {
                let value = format!(
                    "[{}]",
                    read_to_string(&path).unwrap().replace("}\n{", "},{")
                );
                if let Ok(parsed) = serde_json::from_str::<Vec<Ver>>(&value) {
                    let name = parsed.get(0).unwrap().name.clone();
                    let ver: Vec<String> = parsed
                        .into_iter()
                        .filter(|item| !item.yanked)
                        .map(|item| item.vers)
                        .rev()
                        .collect();
                    output.push((name, ver));
                }
            } else if path.is_dir() {
                output.append(&mut read_files_in_directory(&path)?);
            }
        }
    }
    Ok(output)
}

fn main() -> io::Result<()> {
    let dir_path = Path::new("./crates-io-index");

    let output = read_files_in_directory(dir_path)?;
    let mut items: HashMap<char, Vec<(String, Vec<String>)>> = HashMap::new();
    for item in output {
        let f = item
            .0
            .chars()
            .next()
            .unwrap()
            .to_lowercase()
            .collect::<String>()
            .chars()
            .next()
            .unwrap();
        items.entry(f).or_insert_with(|| vec![]).push(item);
    }
    create_dir_all("./index").unwrap();
    for (key, mut value) in items {
        value.sort_by(|(a, _), (b, _)| a.cmp(&b));
        let value = value
            .into_iter()
            .map(|v| serde_json::to_string(&v).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        File::create(format!("./index/{key}.json"))
            .unwrap()
            .write_all(value.as_bytes())
            .unwrap();
    }
    Ok(())
}
