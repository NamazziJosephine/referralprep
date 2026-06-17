// ReferralPrep entry point
// Handles all user interaction and file output. No business logic here.
// The cleaning, validation, and CSV building all live in lib.rs.

use clap::Parser;
use referralprep::{
    completeness_score, csv_header, missing_fields, validate_duration, validate_output_path,
    Field, MedicationDictionary, Referral,
};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

// Optional flags. If no output path is given, the user is prompted for one.
#[derive(Parser, Debug)]
#[command(
    name = "referralprep",
    about = "Turn messy referral notes into a clean, complete clinical CSV, one patient at a time.",
    long_about = None,
    version
)]
struct Args {
    // Where to write the CSV. Created if missing, appended if it already exists.
    #[arg(long, help = "Output CSV file path (e.g. referrals.csv)")]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    print_welcome();

    // Build the medication dictionary, starting with the built-in shorthands
    // and then letting the user add any of their own.
    let mut meds_dict = MedicationDictionary::with_defaults();
    offer_custom_abbreviations(&mut meds_dict);

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
        println!("Enter each field below. Press Enter to leave a text field blank.");
        println!();

        let referral = collect_referral(&meds_dict);
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
}

// At startup, asks whether the user wants to teach the tool any of their own
// medication abbreviations, for example "clari" meaning "clarithromycin".
fn offer_custom_abbreviations(dict: &mut MedicationDictionary) {
    println!();
    println!("The tool already knows some drug shorthands (asp, para, met, amox, ibu).");
    if !confirm("Would you like to add your own medication abbreviations?") {
        return;
    }
    loop {
        let short = read_line("  Abbreviation (e.g. clari): ");
        if short.is_empty() {
            println!("  Skipped (blank abbreviation).");
        } else {
            let full = read_line("  Full medication name (e.g. clarithromycin): ");
            if full.is_empty() {
                println!("  Skipped (blank full name).");
            } else {
                dict.add(&short, &full);
                println!("  Added: {} means {}.", short.trim(), full.trim());
            }
        }
        if !confirm("  Add another abbreviation?") {
            break;
        }
    }
}

// Reads one line of input after showing a prompt, returning the trimmed text.
fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    input.trim().to_string()
}

// Asks a yes or no question. Accepts y, yes, n, no. Defaults to yes on Enter.
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

// Asks for the duration and keeps asking until it passes the 1 to 100 guardrail.
// This is the validation a plain spreadsheet cannot do for you.
fn prompt_duration() -> u32 {
    loop {
        let answer = read_line("Duration in days (1 to 100): ");
        match validate_duration(&answer) {
            Ok(days) => return days,
            Err(message) => println!("  {}", message),
        }
    }
}

// Collects all nine fields from the user, validating the duration and expanding
// medication abbreviations as it goes.
fn collect_referral(meds_dict: &MedicationDictionary) -> Referral {
    let patient_name = read_line("Patient name: ");
    let chief_complaint = read_line("Chief complaint: ");
    let duration_days = prompt_duration();
    let pmh = read_line("Past medical history (PMH): ");
    let medications = read_line("Medications: ");
    let allergies = read_line("Allergies: ");
    let exam_findings = read_line("Exam findings: ");
    let tests = read_line("Tests: ");
    let reason = read_line("Reason for referral: ");

    Referral::build(
        &patient_name,
        &chief_complaint,
        duration_days,
        &pmh,
        &medications,
        &allergies,
        &exam_findings,
        &tests,
        &reason,
        meds_dict,
    )
}

// Shows a cleaned preview plus the completeness score and any missing fields.
fn show_preview(referral: &Referral) {
    println!();
    println!("{}", "-".repeat(60));
    println!("Preview (after cleaning)");
    println!("{}", "-".repeat(60));
    for field in Field::all().iter() {
        let value = referral.value_for(*field);
        let shown = if value.is_empty() { "(blank)".to_string() } else { value };
        println!("  {:<32} {}", format!("{}:", field.prompt_label()), shown);
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
