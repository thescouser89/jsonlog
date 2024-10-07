use clap::Parser;
use colored::ColoredString;
use colored::Colorize;
use serde::Deserialize;
use serde_json::Error;

#[derive(Parser)]
#[command(author, version, about = "PNC JSON log parser", long_about = None)]
struct Args {}

#[derive(Deserialize)]
struct LogLine {
    timestamp: String,
    loggerName: String,
    level: String,
    message: String,
    stackTrace: Option<String>,
    exc_info: Option<String>,
    exception: Option<JavaExceptionMessage>,
}

#[derive(Deserialize)]
struct JavaExceptionMessage {
    message: Option<String>,
    exceptionType: Option<String>,
}

/// Choose an appropriate color for the level and return the ColoredString
fn colored_level(level: &str) -> ColoredString {
    match level {
        "INFO" => level.bold().blue(),
        "ERROR" => level.bold().red(),
        "SEVERE" => level.bold().red(),
        "DEBUG" => level.bold().green(),
        "WARN" => level.bold().bright_yellow(),
        "WARNING" => level.bold().bright_yellow(),
        _ => level.bold(),
    }
}

/// Choose an appropriate color for the message, given the level and return the ColoredString
fn colored_message(level: &str, message: &str) -> ColoredString {
    match level {
        "INFO" => message.blue(),
        "ERROR" => message.red(),
        "SEVERE" => message.red(),
        "DEBUG" => message.green(),
        "WARN" => message.bright_yellow(),
        "WARNING" => message.bright_yellow(),
        _ => message.blue(),
    }
}

/// Pretty print the Logline struct
///
fn print_json(logline: &LogLine) {
    println!(
        "[{}] {} [{}] {}",
        logline.timestamp.bright_white(),
        colored_level(&logline.level),
        logline.loggerName.italic().dimmed(),
        colored_message(&logline.level, &logline.message)
    );

    // print stacktrace if present
    match &logline.stackTrace {
        Some(value) => println!("{}", value.bold().red()),
        None => (),
    }

    // print exc_info if present
    match &logline.exc_info {
        Some(value) => println!("{}", value.bold().yellow()),
        None => (),
    }

    match &logline.exception {
        Some(value) => {
            match &value.exceptionType {
                Some(exception_type) => {
                    print!("{}: ", "Exception type".bright_yellow().bold());
                    println!("{}", exception_type.bold().red()) 
                },
                None => (),
            }
            match &value.message {
                Some(message) => {
                    print!("{}: ", "Message".bright_yellow().bold());
                    println!("{}", message.bold().red().italic())
                },
                None => (),
            }
        }
        None => (),
    }
}

fn main() {
    let _ = Args::parse();

    // read from stdin

    for line in std::io::stdin().lines() {
        let line_stdin = line.unwrap();
        let result: Result<LogLine, Error> = serde_json::from_str(&line_stdin);

        match result {
            // if it can't be parsed to JSON, just print the result as is
            Err(_) => println!("{}", line_stdin),

            // if we can parse to JSON, let's pretty print it!
            Ok(value) => print_json(&value),
        }
    }
}
