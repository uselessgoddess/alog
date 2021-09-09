use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::DirEntry;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

use argparse::{ArgumentParser, Store, StoreFalse, StoreOption, StoreTrue};
use chrono::{Datelike, DateTime, NaiveDate, NaiveDateTime};
use colored::*;
use zip::write::FileOptions;

fn smart_range_name(l: &NaiveDate, r: &NaiveDate) -> String {
    if r.year() - l.year() != 0 {
        format!("[{left:04}-{right:04}]",
                left=l.year(),
                right=r.year())
    } else if r.month() - l.month() != 0 {
        format!("{year:04}-[{left:02}-{right:02}]",
                year=l.year(),
                left=l.month(),
                right=r.month())
    } else {
        format!("{year:04}-{month:02}-[{left:02}-{right:02}]",
                year=l.year(),
                month=l.month(),
                left=l.day(),
                right=r.day())
    }
}

pub trait ColorizeExtensions {
    fn error_text(self) -> ColoredString;
}

impl<'a> ColorizeExtensions for &'a str {
    fn error_text(self) -> ColoredString {
        self.red().italic()
    }
}
impl ColorizeExtensions for String {
    fn error_text(self) -> ColoredString {
        self.as_str().error_text()
    }
}

fn main() {
    let result = real_main();
    match result {
        Ok(_) => {}
        Err(err) => {
            let string = format!("Error: `{}`", err);
            println!("{}", string.red().bold());
            exit(1);
        }
    }
}

fn real_main() -> Result<(), Box<dyn std::error::Error>> {
    // todo!() create Options struct
    let mut path = "".to_string();
    let mut zip_name: Option<String> = None;
    let mut default_format = "%Y-%m-%d.log".to_string();
    let mut safe_paring = false;
    let mut remove_logs = true;
    let base_zip_name = "all_logs.zip".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("TEMP DESCRIPTION");
        ap.refer(&mut path).required()
            .add_argument("PATH", Store,
                          "Path to work folder");
        ap.refer(&mut default_format)
            .add_option(&["--format"], Store,
                        "Format for file parsing");
        ap.refer(&mut safe_paring)
            .add_option(&["--safe-parsing"], StoreTrue,
                        "Use safe parsing");
        ap.refer(&mut remove_logs)
            .add_option(&["--no-remove"], StoreFalse,
                        "No remove log-files");
        ap.refer(&mut zip_name)
            .add_option(&["--zip-name"], StoreOption,
                        "Substitute name instead of generated");
        ap.parse_args_or_exit();
    }

    let paths = fs::read_dir(path.as_str())?;
    let mut files: Vec<(DirEntry, NaiveDate)> = Vec::new();

    for path in paths {
        let path = path?;
        let file_name = path.file_name().to_str().unwrap().to_owned();
        let date_time = NaiveDate::parse_from_str(file_name.as_str(), default_format.as_str());
        if let Ok(date_time) = date_time {
            let date_time = date_time;
            files.push((path, date_time));
        } else {
            if safe_paring {
                let message = format!("not parsing format \"{}\" - use `--safe-parsing`", file_name);
                println!("{}", message.error_text());
                exit(1);
            }
        }
    }

    if files.is_empty() {
        println!("{}", "files not found".error_text());
        exit(0);
    }

    files.sort_by(|(_, a), (_, b)| {
        return a.cmp(&b);
    });

    let open_name = if zip_name.is_some() {
        zip_name.unwrap()
    } else {
        let len = files.len();
        match len {
            _ if {len >= 2} => {
                let l = &files.first().unwrap().1;
                let r = &files.last().unwrap().1;
                format!("{}.zip", smart_range_name(&l, &r))
            }
            1 => {
                let one = &files.first().unwrap().1;
                format!("[{}]", one.to_string())
            }
            _ => { base_zip_name }
        }
    };

    let path = std::path::Path::new(open_name.as_str());
    let file = std::fs::File::create(&path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =  FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for (entry, _) in files.iter()
        .take(files.len() - 1)
    {
        zip.start_file(entry.file_name().to_str().unwrap(), options);

        let path = entry.path();
        let mut buf = String::new();
        let mut file = std::fs::File::open(path.as_path())?;
        file.read_to_string(buf.borrow_mut());
        zip.write_all(buf.as_bytes());
        if remove_logs {
            fs::remove_file(path.as_path());
        }
    }

    Ok(())
}
