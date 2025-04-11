use clap::Parser;
use colored::ColoredString;
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use serde_json::Error;
use strip_ansi_escapes::strip_str;

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
fn print_json(node_name: &str, logline: &LogLine) {


    // if there's a node name, add a space at the end so that the output is prettier
    let final_node_name = if !node_name.is_empty() {
        &format!("{} ", &node_name)
    } else {
        node_name
    };

    println!(
        "{}[{}] {} [{}] {}",
        final_node_name.italic().dimmed(),
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
                }
                None => (),
            }
            match &value.message {
                Some(message) => {
                    print!("{}: ", "Message".bright_yellow().bold());
                    println!("{}", message.bold().red().italic())
                }
                None => (),
            }
        }
        None => (),
    }
}

fn main() {
    let _ = Args::parse();

    // the kubetail output is "[nodename deployment] <rest>"
    let cap = Regex::new(r"^(?:\[(.*?)\])?\s*(.*)").unwrap();


    // read from stdin
    for line in std::io::stdin().lines() {
        let line_stdin = line.unwrap();

        // remove any ascii escape code that adds color
        let line_cleaned = strip_str(&line_stdin);

        if let Some(caps) = cap.captures(&line_cleaned) {
            // the kubetail output is "[nodename deployment] <rest>"
            // try to extract the nodename stuff
            // if we're just using the openshift output, the <rest> should still match
            let node = &caps.get(1).map_or("", |m| m.as_str().split(" ").collect::<Vec<&str>>()[0]);        
            let json_data = &caps.get(2).map_or("", |m| m.as_str());        

            // parse the json if possible
            let result: Result<LogLine, Error> = serde_json::from_str(&json_data);

            match result {
                // if it can't be parsed to JSON, just print the result as is
                Err(_) => println!("{}", &line_cleaned),

                // if we can parse to JSON, let's pretty print it!
                Ok(value) => print_json(&node, &value),
            }
        }
    }
}
