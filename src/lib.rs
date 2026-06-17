// ReferralPrep core library
// All the real logic lives in this file: what fields a referral has, how text
// gets cleaned, how durations are validated, how medication abbreviations are
// expanded, how a CSV row is built, and how completeness is scored.
// main.rs only talks to the user and writes files. It never makes decisions.

use serde::Serialize;
use std::collections::HashMap;

// The largest number of days a course of medication may sensibly last.
// This is the guardrail: anything above this is almost certainly a typo.
pub const MAX_DURATION_DAYS: u32 = 100;

// A referral is made of nine fields. This enum lists each one.
// The order here is both the CSV column order and the order we ask questions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    PatientName,
    ChiefComplaint,
    DurationDays,
    PastMedicalHistory,
    Medications,
    Allergies,
    ExamFindings,
    Tests,
    ReasonForReferral,
}

impl Field {
    // Returns every field in fixed column order.
    pub fn all() -> [Field; 9] {
        [
            Field::PatientName,
            Field::ChiefComplaint,
            Field::DurationDays,
            Field::PastMedicalHistory,
            Field::Medications,
            Field::Allergies,
            Field::ExamFindings,
            Field::Tests,
            Field::ReasonForReferral,
        ]
    }

    // The short, machine-readable name used as the CSV column header.
    pub fn column_header(&self) -> &'static str {
        match self {
            Field::PatientName => "patient_name",
            Field::ChiefComplaint => "chief_complaint",
            Field::DurationDays => "duration_days",
            Field::PastMedicalHistory => "pmh",
            Field::Medications => "medications",
            Field::Allergies => "allergies",
            Field::ExamFindings => "exam_findings",
            Field::Tests => "tests",
            Field::ReasonForReferral => "reason_for_referral",
        }
    }

    // The friendly question shown to the user when we ask for this field.
    pub fn prompt_label(&self) -> &'static str {
        match self {
            Field::PatientName => "Patient name",
            Field::ChiefComplaint => "Chief complaint",
            Field::DurationDays => "Duration in days (1 to 100)",
            Field::PastMedicalHistory => "Past medical history (PMH)",
            Field::Medications => "Medications",
            Field::Allergies => "Allergies",
            Field::ExamFindings => "Exam findings",
            Field::Tests => "Tests",
            Field::ReasonForReferral => "Reason for referral",
        }
    }

    // Whether a blank value here should trigger a completeness warning.
    pub fn is_required(&self) -> bool {
        true
    }

    // The patient name is special: we never run abbreviation expansion on it,
    // so a name like "Asp" is never turned into "aspirin".
    pub fn is_name(&self) -> bool {
        matches!(self, Field::PatientName)
    }
}

// Validates a duration entered as text. It must be a whole number from 1 to 100.
// This is the guardrail your tool enforces that a plain spreadsheet cannot.
pub fn validate_duration(raw: &str) -> Result<u32, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Duration cannot be empty.".to_string());
    }
    match trimmed.parse::<u32>() {
        Ok(0) => Err("Duration must be at least 1 day.".to_string()),
        Ok(days) if days > MAX_DURATION_DAYS => Err(format!(
            "Duration cannot exceed {} days. You entered {}.",
            MAX_DURATION_DAYS, days
        )),
        Ok(days) => Ok(days),
        Err(_) => Err(format!(
            "Duration must be a whole number of days, not '{}'.",
            trimmed
        )),
    }
}

// The set of medication abbreviations the tool knows.
// It starts with a few built-in ones and can be extended by the user.
#[derive(Debug, Clone)]
pub struct MedicationDictionary {
    entries: HashMap<String, String>,
}

impl MedicationDictionary {
    // Creates a dictionary pre-loaded with a few common medication shorthands.
    pub fn with_defaults() -> MedicationDictionary {
        let mut entries = HashMap::new();
        entries.insert("asp".to_string(), "aspirin".to_string());
        entries.insert("para".to_string(), "paracetamol".to_string());
        entries.insert("met".to_string(), "metformin".to_string());
        entries.insert("amox".to_string(), "amoxicillin".to_string());
        entries.insert("ibu".to_string(), "ibuprofen".to_string());
        MedicationDictionary { entries }
    }

    // Adds or overrides one abbreviation. The short form is stored lowercased
    // so matching later is not affected by capitalisation.
    pub fn add(&mut self, short: &str, full: &str) {
        let key = short.trim().to_lowercase();
        let value = full.trim().to_string();
        if !key.is_empty() && !value.is_empty() {
            self.entries.insert(key, value);
        }
    }

    // Looks up one word. If it is a known abbreviation, returns the full name.
    fn expand_word(&self, word: &str) -> String {
        // Compare on letters only so "asp," and "Asp" both match "asp".
        let key: String = word.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect();
        match self.entries.get(&key) {
            Some(full) => full.clone(),
            None => word.to_string(),
        }
    }

    // Expands every medication abbreviation in a piece of text.
    pub fn expand_medications(&self, text: &str) -> String {
        text.split(' ')
            .map(|word| self.expand_word(word))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

// One fully structured referral. Duration is stored as a validated number.
#[derive(Debug, Clone, Serialize)]
pub struct Referral {
    pub patient_name: String,
    pub chief_complaint: String,
    pub duration_days: u32,
    pub pmh: String,
    pub medications: String,
    pub allergies: String,
    pub exam_findings: String,
    pub tests: String,
    pub reason_for_referral: String,
}

impl Referral {
    // Builds a referral. The caller has already validated the duration and
    // supplies the medication dictionary so shorthands get expanded.
    pub fn build(
        patient_name: &str,
        chief_complaint: &str,
        duration_days: u32,
        pmh: &str,
        medications: &str,
        allergies: &str,
        exam_findings: &str,
        tests: &str,
        reason_for_referral: &str,
        meds_dict: &MedicationDictionary,
    ) -> Referral {
        Referral {
            patient_name: tidy_name(patient_name),
            chief_complaint: clean_text(chief_complaint),
            duration_days,
            pmh: clean_text(pmh),
            // Medications get general cleaning first, then drug-name expansion.
            medications: meds_dict.expand_medications(&clean_text(medications)),
            allergies: clean_text(allergies),
            exam_findings: clean_text(exam_findings),
            tests: clean_text(tests),
            reason_for_referral: clean_text(reason_for_referral),
        }
    }

    // Returns the stored value for one field as text, so we can loop generically.
    pub fn value_for(&self, field: Field) -> String {
        match field {
            Field::PatientName => self.patient_name.clone(),
            Field::ChiefComplaint => self.chief_complaint.clone(),
            Field::DurationDays => self.duration_days.to_string(),
            Field::PastMedicalHistory => self.pmh.clone(),
            Field::Medications => self.medications.clone(),
            Field::Allergies => self.allergies.clone(),
            Field::ExamFindings => self.exam_findings.clone(),
            Field::Tests => self.tests.clone(),
            Field::ReasonForReferral => self.reason_for_referral.clone(),
        }
    }

    // Turns this referral into one CSV line, escaping each field safely.
    pub fn to_csv_row(&self) -> String {
        Field::all()
            .iter()
            .map(|f| escape_csv_field(&self.value_for(*f)))
            .collect::<Vec<String>>()
            .join(",")
    }
}

// Cleans a name: only collapse the spacing, never change the words.
fn tidy_name(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<&str>>().join(" ")
}

// Cleans a normal text field: squeeze messy whitespace into single spaces,
// then expand common clinical abbreviations like OD and NKDA.
pub fn clean_text(raw: &str) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<&str>>().join(" ");
    expand_abbreviations(&collapsed)
}

// Expands general clinical shorthand (not drug names) to full wording.
fn expand_abbreviations(text: &str) -> String {
    let replacements: [(&str, &str); 8] = [
        ("od", "once daily"),
        ("bd", "twice daily"),
        ("tds", "three times daily"),
        ("qds", "four times daily"),
        ("prn", "as needed"),
        ("nkda", "no known drug allergies"),
        ("pmh", "past medical history"),
        ("hx", "history"),
    ];

    text.split(' ')
        .map(|word| {
            let trimmed: String = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect();
            for (short, long) in replacements.iter() {
                if trimmed == *short {
                    return long.to_string();
                }
            }
            word.to_string()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// Makes a single value safe for CSV. If it contains a comma, quote, or newline,
// we wrap it in quotes and double any inner quotes, the standard CSV rule.
pub fn escape_csv_field(value: &str) -> String {
    let needs_quoting = value.contains(',') || value.contains('"') || value.contains('\n');
    if needs_quoting {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

// Builds the CSV header line from every column name in order.
pub fn csv_header() -> String {
    Field::all()
        .iter()
        .map(|f| f.column_header().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

// Returns the required fields the user left empty. Duration is numeric and
// always set, so it never appears here.
pub fn missing_fields(referral: &Referral) -> Vec<Field> {
    Field::all()
        .iter()
        .filter(|f| f.is_required() && referral.value_for(**f).trim().is_empty())
        .copied()
        .collect()
}

// Gives a completeness score from 0 to 100 based on filled required fields.
pub fn completeness_score(referral: &Referral) -> u32 {
    let total = Field::all().iter().filter(|f| f.is_required()).count() as u32;
    if total == 0 {
        return 100;
    }
    let missing = missing_fields(referral).len() as u32;
    let filled = total - missing;
    (filled * 100) / total
}

// Checks the chosen output path is not empty before we use it.
pub fn validate_output_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Output path cannot be empty.".to_string());
    }
    Ok(trimmed.to_string())
}
