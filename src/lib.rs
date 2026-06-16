// ReferralPrep core library
// All the real logic lives in this file: what fields a referral has, how text
// gets cleaned, how a CSV row is built, and how completeness is scored.
// main.rs only talks to the user and writes files. It never makes decisions.

use serde::Serialize;

// A referral is made of nine fields. This enum lists each one.
// The order here is important: it is both the order of columns in the CSV
// and the order the user is asked the questions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    PatientName,
    ChiefComplaint,
    Duration,
    PastMedicalHistory,
    Medications,
    Allergies,
    ExamFindings,
    Tests,
    ReasonForReferral,
}

impl Field {
    // Returns every field in fixed column order.
    // Whenever we loop over a referral, we loop over this array.
    pub fn all() -> [Field; 9] {
        [
            Field::PatientName,
            Field::ChiefComplaint,
            Field::Duration,
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
            Field::Duration => "duration",
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
            Field::Duration => "Duration",
            Field::PastMedicalHistory => "Past medical history (PMH)",
            Field::Medications => "Medications",
            Field::Allergies => "Allergies",
            Field::ExamFindings => "Exam findings",
            Field::Tests => "Tests",
            Field::ReasonForReferral => "Reason for referral",
        }
    }

    // Whether a blank value in this field should trigger a completeness warning.
    // Every field counts as required for a complete referral.
    pub fn is_required(&self) -> bool {
        true
    }

    // The patient name is a special case: we do not run abbreviation expansion
    // on it, because a name like "Od" must never become "once daily".
    pub fn is_name(&self) -> bool {
        matches!(self, Field::PatientName)
    }
}

// One fully structured referral: the cleaned value for each of the nine fields.
#[derive(Debug, Clone, Serialize)]
pub struct Referral {
    pub patient_name: String,
    pub chief_complaint: String,
    pub duration: String,
    pub pmh: String,
    pub medications: String,
    pub allergies: String,
    pub exam_findings: String,
    pub tests: String,
    pub reason_for_referral: String,
}

impl Referral {
    // Builds a referral from nine raw values given in field order.
    // The name is only trimmed; the other fields also get abbreviations expanded.
    pub fn from_values(values: &[String; 9]) -> Referral {
        Referral {
            patient_name: tidy_name(&values[0]),
            chief_complaint: clean_text(&values[1]),
            duration: clean_text(&values[2]),
            pmh: clean_text(&values[3]),
            medications: clean_text(&values[4]),
            allergies: clean_text(&values[5]),
            exam_findings: clean_text(&values[6]),
            tests: clean_text(&values[7]),
            reason_for_referral: clean_text(&values[8]),
        }
    }

    // Returns the stored value for one field, so we can loop over fields generically.
    pub fn value_for(&self, field: Field) -> &str {
        match field {
            Field::PatientName => &self.patient_name,
            Field::ChiefComplaint => &self.chief_complaint,
            Field::Duration => &self.duration,
            Field::PastMedicalHistory => &self.pmh,
            Field::Medications => &self.medications,
            Field::Allergies => &self.allergies,
            Field::ExamFindings => &self.exam_findings,
            Field::Tests => &self.tests,
            Field::ReasonForReferral => &self.reason_for_referral,
        }
    }

    // Turns this referral into a single CSV line, escaping each field safely.
    pub fn to_csv_row(&self) -> String {
        Field::all()
            .iter()
            .map(|f| escape_csv_field(self.value_for(*f)))
            .collect::<Vec<String>>()
            .join(",")
    }
}

// Cleans a name: only collapse the spacing, never touch the words themselves.
fn tidy_name(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<&str>>().join(" ")
}

// Cleans a normal text field in two steps:
// first squeeze any messy whitespace into single spaces,
// then expand common clinical abbreviations to full words.
pub fn clean_text(raw: &str) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<&str>>().join(" ");
    expand_abbreviations(&collapsed)
}

// Looks at each word and, if it matches a known abbreviation, swaps in the
// full wording. We compare on letters only, so "OD," and "od" both match.
fn expand_abbreviations(text: &str) -> String {
    // The shorthand on the left becomes the clear wording on the right.
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
            // Strip punctuation and lowercase the word before comparing.
            let trimmed: String = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect();

            // If this word is a known abbreviation, return the full form.
            for (short, long) in replacements.iter() {
                if trimmed == *short {
                    return long.to_string();
                }
            }

            // Otherwise keep the original word untouched.
            word.to_string()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// Makes a single value safe to put in a CSV.
// If it contains a comma, quote, or line break, we wrap it in quotes and
// double any quotes inside, which is the standard CSV escaping rule.
pub fn escape_csv_field(value: &str) -> String {
    let needs_quoting = value.contains(',') || value.contains('"') || value.contains('\n');
    if needs_quoting {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

// Builds the header line of the CSV from every column name in order.
pub fn csv_header() -> String {
    Field::all()
        .iter()
        .map(|f| f.column_header().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

// Returns the list of required fields that the user left empty.
// main.rs uses this to warn about incomplete notes.
pub fn missing_fields(referral: &Referral) -> Vec<Field> {
    Field::all()
        .iter()
        .filter(|f| f.is_required() && referral.value_for(**f).trim().is_empty())
        .copied()
        .collect()
}

// Gives a completeness score from 0 to 100 based on how many required fields
// are filled. A referral with no blanks scores 100.
pub fn completeness_score(referral: &Referral) -> u32 {
    let total = Field::all().iter().filter(|f| f.is_required()).count() as u32;
    if total == 0 {
        return 100;
    }
    let missing = missing_fields(referral).len() as u32;
    let filled = total - missing;
    (filled * 100) / total
}

// Checks that the chosen output path is not empty before we try to use it.
pub fn validate_output_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Output path cannot be empty.".to_string());
    }
    Ok(trimmed.to_string())
}
