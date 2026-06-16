// ReferralPrep entry point
// Handles all user interaction and file output. No business logic here.
// The cleaning, CSV building, and completeness checks all live in lib.rs.

use clap::Parser;
use referralprep::{
    completeness_score, csv_header, missing_fields, validate_output_path, Field, Referral,
};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

// Optional flags. If no output path is given, the user is prompted for one.
#[derive(Parser, Debug)]
#[command(
    name = "referralprep",
    about = "Turn messy referral notes into a clean, complete clinical CSV, one note at a time.",
    long_about = None,
    version
)]
struct Args {
    // Where to write the CSV. The file is created if missing, appended if present.
    #[arg(long, help = "Output CSV file path (e.g. referrals.csv)")]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    print_welcome();

    let output_path = match args.output {
        Some(p) => match validate_output_path(&p) {
            Ok(valid) => valid,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        None => prompt_output_path(),
    };

    ensure_header(&output_path);

    let mut added = 0;
    loop {
        println!();
        println!("{}", "=".repeat(60));
        println!("New referral note");
        println!("{}", "=".repeat(60));
        println!("Enter each field below. Press Enter to leave a field blank.");
        println!();

        let referral = collect_referral();
        show_preview(&referral);

        if confirm("Add this referral to the CSV?") {
            append_referral(&output_path, &referral);
            added += 1;
            println!("Saved. Total referrals added this session: {}.", added);
        } else {
            println!("Discarded. Nothing was written.");
        }

        if !confirm("Add another referral?") {
            break;
        }
    }

    println!();
    println!("Done. {} referral(s) written to {}.", added, output_path);
    println!("Open the file in Excel, Google Sheets, or any spreadsheet tool.");
}

// Prints a friendly welcome banner explaining what the tool does.
fn print_welcome() {
    println!();
    println!("ReferralPrep");
    println!("Turn messy referral notes into a clean, complete clinical CSV.");
    println!("You will be asked for each field one at a time. Let us begin.");
}

// Reads one line of input after showing a prompt, returning the trimmed text.
fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    input.trim().to_string()
}

// Asks a yes or no question and returns true for yes.
// Accepts y, yes, n, no in any capitalisation. Defaults to yes on empty input.
fn confirm(question: &str) -> bool {
    loop {
        let answer = read_line(&format!("{} [Y/n]: ", question)).to_lowercase();
        match answer.as_str() {
            "" | "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("  Please answer y or n."),
        }
    }
}

// Prompts for the output file path until a non-empty value is given.
fn prompt_output_path() -> String {
    loop {
        let answer = read_line("Output CSV file name [referrals.csv]: ");
        let chosen = if answer.is_empty() {
            "referrals.csv".to_string()
        } else {
            answer
        };
        match validate_output_path(&chosen) {
            Ok(valid) => return valid,
            Err(e) => println!("  {}", e),
        }
    }
}

// Asks the user for all nine fields, one at a time, in column order.
// We store each answer in an array, then hand it to the library to clean.
fn collect_referral() -> Referral {
    let mut values: [String; 9] = Default::default();
    for (i, field) in Field::all().iter().enumerate() {
        // Build the question from the field's friendly label, e.g. "Duration: ".
        let prompt = format!("{}: ", field.prompt_label());
        values[i] = read_line(&prompt);
    }
    Referral::from_values(&values)
}

// Shows a cleaned preview of the referral plus its completeness score and any
// missing fields, so the user can decide whether to save or re-edit.
fn show_preview(referral: &Referral) {
    println!();
    println!("{}", "-".repeat(60));
    println!("Preview (after cleaning)");
    println!("{}", "-".repeat(60));
    for field in Field::all().iter() {
        let value = referral.value_for(*field);
        let shown = if value.is_empty() { "(blank)" } else { value };
        println!("  {:<28} {}", format!("{}:", field.prompt_label()), shown);
    }

    let score = completeness_score(referral);
    let missing = missing_fields(referral);
    println!();
    println!("  Completeness: {}%", score);
    if missing.is_empty() {
        println!("  All required fields present.");
    } else {
        let names: Vec<&str> = missing.iter().map(|f| f.prompt_label()).collect();
        println!("  Missing: {}", names.join(", "));
    }
    println!();
}

// Creates the output file with a header row if it does not already exist.
fn ensure_header(path: &str) {
    if Path::new(path).exists() && file_has_content(path) {
        return;
    }
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            let _ = writeln!(file, "{}", csv_header());
        }
        Err(e) => {
            eprintln!("Could not create {}: {}", path, e);
            std::process::exit(1);
        }
    }
}

// Returns true if the file exists and is not empty.
fn file_has_content(path: &str) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut buffer = String::new();
        if file.read_to_string(&mut buffer).is_ok() {
            return !buffer.trim().is_empty();
        }
    }
    false
}

// Appends one referral as a CSV row to the output file.
fn append_referral(path: &str, referral: &Referral) {
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            if writeln!(file, "{}", referral.to_csv_row()).is_err() {
                eprintln!("Could not write to {}.", path);
            }
        }
        Err(e) => {
            eprintln!("Could not open {}: {}", path, e);
        }
    }
}
