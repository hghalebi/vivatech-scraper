// Partners/exhibitor scraping module for VivaTech

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

// Constants
pub const PARTNERS_URL: &str = "https://vivatechnology.com/partners";
pub const DEFAULT_PARTNERS_OUTPUT: &str = "vivatech_partners_2025.csv";

// Partner data model
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Partner {
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub logo_url: String,
}

// CSV output format
#[derive(Debug, Serialize)]
pub struct PartnerRecord {
    #[serde(rename = "CompanyName")]
    company_name: String,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Country")]
    country: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Website")]
    website: String,
    #[serde(rename = "LogoURL")]
    logo_url: String,
}

// Extract partner data from HTML - looks for JSON array
pub fn extract_partners_from_html(html_content: &str) -> Result<Vec<Partner>> {
    if let Some(start_idx) = html_content.find(r#"[{\"id\":\""#) {
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in html_content[start_idx..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => escape_next = true,
                '"' if !escape_next => in_string = !in_string,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        let json_str = &html_content[start_idx..=start_idx + i];
                        let unescaped = json_str.replace(r#"\""#, r#"""#);
                        let final_json = unescape_unicode(&unescaped);

                        if let Ok(json_value) =
                            serde_json::from_str::<serde_json::Value>(&final_json)
                        {
                            if let Some(array) = json_value.as_array() {
                                return Ok(extract_partners_from_json_array(array));
                            }
                        }
                        break;
                    }
                }
                _ => {}
            }

            if i > 50_000_000 {
                break;
            } // Safety limit
        }
    }

    Err(anyhow::anyhow!("No partner data found"))
}

// Extract partners from parsed JSON array
fn extract_partners_from_json_array(array: &[serde_json::Value]) -> Vec<Partner> {
    let mut partners = Vec::new();

    for item in array {
        if let Some(obj) = item.as_object() {
            // Check if this looks like a partner/exhibitor object
            if let (Some(name_val), Some(type_val)) = (obj.get("name"), obj.get("type")) {
                if let (Some(name), Some(type_str)) = (name_val.as_str(), type_val.as_str()) {
                    // Only include partners and startups
                    if type_str.contains("partner") || type_str == "startup" {
                        let partner = Partner {
                            name: name.to_string(),
                            category: type_str.to_string(),
                            country: obj
                                .get("key_figures")
                                .and_then(|kf| kf.get("city"))
                                .and_then(|c| c.as_str())
                                .map_or_else(
                                    || extract_country_from_name(name),
                                    |city| extract_country_from_city(city).to_string(),
                                ),
                            description: obj
                                .get("desc")
                                .or_else(|| obj.get("short_desc"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            website: obj
                                .get("website")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            logo_url: obj
                                .get("logo")
                                .and_then(|logo| logo.get("u"))
                                .and_then(|u| u.as_str())
                                .unwrap_or("")
                                .to_string(),
                        };

                        // Avoid duplicates
                        if !partners.iter().any(|p: &Partner| p.name == partner.name) {
                            partners.push(partner);
                        }
                    }
                }
            }
        }
    }

    log::info!("Extracted {} partners from JSON array", partners.len());
    partners
}

// Map common cities to countries
fn extract_country_from_city(city: &str) -> &str {
    match city {
        s if s.contains("Paris") => "France",
        s if s.contains("London") => "UK",
        s if s.contains("Berlin") => "Germany",
        s if s.contains("Tokyo") => "Japan",
        s if s.contains("New York") || s.contains("San Francisco") => "USA",
        s if s.contains("Beijing") || s.contains("Shanghai") => "China",
        s if s.contains("Mumbai") || s.contains("Bangalore") => "India",
        s if s.contains("Toronto") || s.contains("Montreal") => "Canada",
        _ => "",
    }
}

// Try to extract country from company name (e.g., "Company - France")
fn extract_country_from_name(name: &str) -> String {
    if let Some(dash_pos) = name.rfind(" - ") {
        let potential_country = name[dash_pos + 3..].trim();
        if is_likely_country(potential_country) {
            return potential_country.to_string();
        }
    }

    // Check for common country names in company name
    let countries = [
        ("France", "France"),
        ("USA", "USA"),
        ("United States", "USA"),
        ("UK", "UK"),
        ("United Kingdom", "UK"),
        ("Germany", "Germany"),
        ("Japan", "Japan"),
        ("China", "China"),
        ("India", "India"),
        ("Canada", "Canada"),
    ];

    let name_upper = name.to_uppercase();
    for (pattern, country) in &countries {
        if name_upper.contains(&pattern.to_uppercase()) {
            return (*country).to_string();
        }
    }

    String::new()
}

// Check if text is likely a country name
fn is_likely_country(text: &str) -> bool {
    let countries = [
        "France",
        "USA",
        "United States",
        "UK",
        "United Kingdom",
        "Germany",
        "Japan",
        "China",
        "India",
        "Canada",
        "Spain",
        "Italy",
        "Netherlands",
        "Belgium",
        "Switzerland",
        "Austria",
        "Australia",
        "New Zealand",
        "Singapore",
        "Korea",
        "Brazil",
        "Mexico",
        "Argentina",
        "Chile",
        "Poland",
        "Czech Republic",
        "Hungary",
        "Romania",
        "Greece",
        "Portugal",
        "Ireland",
        "Scotland",
        "Wales",
        "Sweden",
        "Norway",
        "Denmark",
        "Finland",
        "Russia",
        "Ukraine",
        "Turkey",
        "Israel",
        "UAE",
        "Saudi Arabia",
        "Egypt",
        "South Africa",
        "Nigeria",
        "Kenya",
        "Morocco",
        "Algeria",
        "Tunisia",
        "Albania",
        "Armenia",
        "Bangladesh",
    ];

    countries
        .iter()
        .any(|&country| text.eq_ignore_ascii_case(country))
}

// Convert to CSV format
pub fn convert_to_partner_records(partners: Vec<Partner>) -> Vec<PartnerRecord> {
    partners
        .into_iter()
        .map(|partner| PartnerRecord {
            company_name: partner.name,
            category: partner.category,
            country: partner.country,
            description: partner.description,
            website: partner.website,
            logo_url: partner.logo_url,
        })
        .collect()
}

// Write to CSV file
pub fn write_partners_to_csv(records: &[PartnerRecord], output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(file);

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    Ok(())
}

// Unescape Unicode sequences
fn unescape_unicode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    'u' => {
                        let hex_chars: String = chars.by_ref().take(4).collect();
                        if hex_chars.len() == 4 {
                            if let Ok(code_point) = u32::from_str_radix(&hex_chars, 16) {
                                if let Some(unicode_char) = char::from_u32(code_point) {
                                    result.push(unicode_char);
                                    continue;
                                }
                            }
                        }
                        // If parsing failed, add the original sequence
                        result.push('\\');
                        result.push('u');
                        result.push_str(&hex_chars);
                    }
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    _ => {
                        result.push('\\');
                        result.push(next_ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}
