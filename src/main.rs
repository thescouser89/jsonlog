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
    #[serde(rename = "loggerName")]
    logger_name: String,
    level: String,
    message: String,
    #[serde(rename = "stackTrace")]
    stack_trace: Option<String>,
    exc_info: Option<String>,
    exception: Option<JavaExceptionMessage>,
}

#[derive(Deserialize)]
struct JavaExceptionMessage {
    message: Option<String>,
    #[serde(rename = "exceptionType")]
    exception_type: Option<String>,
    #[serde(rename = "causedBy")]
    caused_by: Option<Box<CausedByWrapper>>,
    frames: Option<Vec<StackFrame>>,
}

#[derive(Deserialize)]
struct CausedByWrapper {
    exception: JavaExceptionMessage,
}

#[derive(Deserialize)]
struct StackFrame {
    class: Option<String>,
    method: Option<String>,
    line: Option<i64>,
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

fn print_exception(exception: &JavaExceptionMessage, is_caused_by: bool) {
    let prefix = if is_caused_by { "Caused by" } else { "Exception type" };
    if let Some(exception_type) = &exception.exception_type {
        print!("{}: ", prefix.bright_yellow().bold());
        println!("{}", exception_type.bold().red());
    }
    if let Some(message) = &exception.message {
        print!("{}: ", "Message".bright_yellow().bold());
        println!("{}", message.bold().red().italic());
    }
    if let Some(frames) = &exception.frames {
        for frame in frames {
            let class = frame.class.as_deref().unwrap_or("?");
            let method = frame.method.as_deref().unwrap_or("?");
            let line = frame.line.map_or("?".to_string(), |l| l.to_string());
            println!("    {} {}.{}:{}", "at".dimmed(), class.red(), method.red(), line.red());
        }
    }
    if let Some(caused_by) = &exception.caused_by {
        print_exception(&caused_by.exception, true);
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
        logline.logger_name.italic().dimmed(),
        colored_message(&logline.level, &logline.message)
    );

    // print stacktrace if present
    match &logline.stack_trace {
        Some(value) => println!("{}", value.bold().red()),
        None => (),
    }

    // print exc_info if present
    match &logline.exc_info {
        Some(value) => println!("{}", value.bold().yellow()),
        None => (),
    }

    if let Some(value) = &logline.exception {
        print_exception(value, false);
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
