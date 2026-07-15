use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Serialize;
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::{fmt, fs};

#[derive(Debug, Clone, Serialize)]
pub struct HttpError {
    code: u16,
    message: String,
}

impl HttpError {
    pub fn new(code: u16, message: String) -> HttpError {
        HttpError { code, message }
    }

    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Http error [{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for HttpError {}

// Thread local safe variable where we store the last backtrace
thread_local! {
    static LAST_BACKTRACE: RefCell<Option<Backtrace>> = RefCell::new(None)
}

pub fn register_panic_hook(root: PathBuf) {
    // Make sure pattern is in the top scope of this function so it's only compiled once.
    let pattern = Regex::new(r"\Aapp\.\d{10}.log\z").unwrap();
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        LAST_BACKTRACE.with(|backtrace| {
            *backtrace.borrow_mut() = Some(Backtrace::force_capture());

            // todo use parent project root relative route
            let mut paths = fs::read_dir(root.join("storage/logs"))
                .unwrap_or_else(|_| {
                    fs::create_dir(root.join("storage/logs")).expect("failed to create dir");
                    fs::read_dir(root.join("storage/logs")).unwrap()
                })
                .map(|p| String::from(p.unwrap().file_name().to_str().unwrap()))
                .filter(|n| pattern.is_match(n))
                .collect::<Vec<String>>();

            let mut file;

            if paths.len() == 0 {
                // Create log file if none exist
                file = create_log_file().unwrap();
            } else {
                // Get latest file from paths
                paths.sort();
                let file_name = paths.iter().last().unwrap();
                file = OpenOptions::new()
                    .append(true)
                    .open(format!("storage/logs/{file_name}"))
                    .unwrap();

                // Create new log file if latest one >= 25mb
                let metadata = file.metadata().unwrap();
                let mb = metadata.len() / 1024 / 1024;
                if mb >= 25 {
                    file = create_log_file().unwrap();
                }

                if paths.len() >= 10 {
                    // Delete earliest log file
                    let file_name = paths.iter().nth(0).unwrap();
                    fs::remove_file(format!("storage/logs/{file_name}"))
                        .expect(format!("Failed to remove file {file_name}.").as_str());
                }
            }

            // Attempt to write to file.
            match file.lock() {
                Ok(_) => {
                    // todo add date/time to log
                    let mut msg = info.to_string();
                    msg.push('\n');
                    file.write_all(msg.to_string().as_bytes()).unwrap();
                }
                Err(error) => {
                    eprintln!("Failed to log panic.");
                    dbg!(error);
                }
            };
        });

        // Run default hook after logging
        default(info);
    }));
}

fn create_log_file() -> Result<File, std::io::Error> {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.format("%s");

    Ok(OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("storage/logs/app.{timestamp}.log"))?)
}
