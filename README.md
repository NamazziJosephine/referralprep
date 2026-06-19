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

There are three downloads, one per system. Pick the file that matches your computer.

| Your system | File to download |
|---|---|
| Windows | `referralprep-windows-x86_64.exe` |
| Mac (Apple Silicon: M1, M2, M3) | `referralprep-macos-arm64` |
| Linux (Intel/AMD 64-bit) | `referralprep-linux-x86_64` |
| Linux (ARM 64-bit, aarch64) | `referralprep-linux-aarch64` |

Get them from the [website](https://NamazziJosephine.github.io/referralprep) or the [Releases page](../../releases/latest).

(Using an older Intel Mac, or a system not listed here? Skip to "Build from source" below.)

### What to do after downloading

**On Windows**

1. The file lands in your Downloads folder.
2. Open PowerShell (search "PowerShell" in the Start menu).
3. Go to your Downloads and run the program:

```powershell
cd ~\Downloads
.\referralprep-windows-x86_64.exe
```

If Windows shows a blue "Windows protected your PC" box, click "More info" then "Run anyway". This appears because the program is not signed by a paid certificate, which is normal for student tools.

**On Mac or Linux**

1. The file lands in your Downloads folder.
2. Open the Terminal app.
3. Mark the file as runnable (a one-time step), then run it. Use whichever file name you downloaded:

On Linux, pick the file matching your processor (most laptops and desktops are x86_64; ARM servers and some devices are aarch64):

```bash
cd ~/Downloads
chmod +x referralprep-linux-x86_64
./referralprep-linux-x86_64
```

On a Mac (Apple Silicon):

```bash
cd ~/Downloads
chmod +x referralprep-macos-arm64
./referralprep-macos-arm64
```

On a Mac you may see a warning that the developer cannot be verified. Open System Settings, go to Privacy and Security, and click "Open Anyway", then run the command again.

### Using the program

Once it starts, it interviews you. There is nothing to learn: it asks one question at a time and you type the answer, then press Enter.

```
ReferralPrep
Turn messy referral notes into a clean, complete clinical CSV.

Would you like to add your own medication abbreviations? [Y/n]: n
Output CSV file name [referrals.csv]: referrals.csv

Patient name: John Smith
Chief complaint: chest pain
Duration in days (1 to 100): 3
...
Add this referral to the CSV? [Y/n]: y
Add another referral? [Y/n]: n
```

When you finish, open the `referrals.csv` file it created in Excel or Google Sheets. Every patient is one clean row.

### Build from source

Intel Mac users and anyone whose system is not listed above can build the tool this way. It works on any operating system.

Requires [Rust](https://rustup.rs).

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
