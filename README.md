# ReferralPrep

A command-line tool that turns messy referral notes into a clean, complete clinical CSV, one note at a time.

## The problem

Referral letters arrive messy and incomplete: missing durations, no allergy line, medications written as shorthand like "aspirin OD". Turning that into a usable database row means cleaning it by hand, field by field. ReferralPrep does the cleanup and refuses to let a required field go missing unnoticed.

## What it does

- Interviews you for each field of a referral, starting with the patient name, one at a time
- Cleans the text: trims whitespace, collapses spacing, expands common clinical abbreviations (OD, BD, TDS, PRN, NKDA, and more)
- Checks completeness and flags any missing required field with a score
- Appends each note as a tidy row to a CSV you can open in Excel or Google Sheets

## Installation

### Download a prebuilt binary

Go to the [website](https://NamazziJosephine.github.io/referralprep) or the [Releases page](../../releases/latest) and download the file for your system:

| System | File |
|---|---|
| Windows 64-bit | `referralprep-windows-x86_64.exe` |
| macOS Apple Silicon | `referralprep-macos-arm64` |
| macOS Intel | `referralprep-macos-x86_64` |
| Linux 64-bit | `referralprep-linux-x86_64` |

On macOS and Linux, make it executable first:

```bash
chmod +x referralprep-macos-arm64
./referralprep-macos-arm64
```

On Windows, run it from PowerShell:

```powershell
.\referralprep-windows-x86_64.exe
```

### Build from source

Requires [Rust](https://rustup.rs). Works on any operating system.

```bash
git clone https://github.com/NamazziJosephine/referralprep
cd referralprep
cargo build --release
./target/release/referralprep
```

## Usage

Run it with no arguments and it interviews you:

```
ReferralPrep
Turn messy referral notes into a clean, complete clinical CSV.

Output CSV file name [referrals.csv]: referrals.csv

Chief complaint: chest pain
Duration: 3 days
Past medical history (PMH): hypertension, diabetes
Medications: aspirin OD, metformin BD
Allergies: NKDA
Exam findings: BP 150/95
Tests: ECG
Reason for referral: cardiology assessment

Preview (after cleaning)
  Medications:  aspirin once daily, metformin twice daily
  Allergies:    no known drug allergies
  Completeness: 100%
  All required fields present.
Add this referral to the CSV? [Y/n]: y
```

You can also set the output file with a flag:

```bash
referralprep --output cardiology_referrals.csv
```

## Website

[https://NamazziJosephine.github.io/referralprep](https://NamazziJosephine.github.io/referralprep)

The website includes a live browser demo that previews what the tool does.

## License

MIT
