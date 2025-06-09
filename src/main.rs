// VivaTech conference speaker scraper
// Extracts speaker data from embedded JSON in the website

mod partners;

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Constants
const DEFAULT_OUTPUT_FILE: &str = "vivatech_speakers_2025_extended.csv";
const TARGET_URL: &str = "https://vivatechnology.com/speakers";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

#[derive(Parser)]
#[command(
    name = "vivatech-scraper",
    version,
    about = "🔍 Scrapes VivaTech conference data",
    long_about = "A robust web scraper that extracts speaker and partner data from the VivaTech conference website.\n\
                  It targets embedded JSON data for reliability and exports the results to CSV format."
)]
struct Cli {
    /// What to scrape: 'speakers' or 'partners'
    #[arg(value_enum, default_value = "speakers")]
    target: ScrapeTarget,

    /// Output CSV file path (defaults depend on target)
    #[arg(short, long)]
    output: Option<String>,

    /// Enable verbose logging for debugging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Override the target URL (mainly for testing purposes)
    #[arg(long, hide = true)]
    url: Option<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ScrapeTarget {
    Speakers,
    Partners,
}

// Speaker data model matching JSON structure
#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
struct Speaker {
    id: String,
    firstname: String,
    lastname: String,
    #[serde(default)]
    email: String,
    #[serde(rename = "jobTitle")]
    job_title: String,
    company: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    themes: Vec<String>,
    image: Option<Image>,
    #[serde(rename = "hasBio", default)]
    has_bio: bool,
    #[serde(rename = "hasSessions", default)]
    has_sessions: bool,
    #[serde(rename = "isOfficial", default)]
    is_official: bool,
    #[serde(rename = "isPartner", default)]
    is_partner: bool,
    #[serde(default)]
    top: bool,
    #[serde(default)]
    communication_manager: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Image {
    #[serde(default)]
    s: String,
    #[serde(default)]
    t: String,
    #[serde(default)]
    l: String,
    u: String,
}

// CSV output format
#[derive(Debug, Serialize)]
#[allow(clippy::struct_excessive_bools)]
struct SpeakerRecord {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "FirstName")]
    first_name: String,
    #[serde(rename = "LastName")]
    last_name: String,
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "JobTitle")]
    job_title: String,
    #[serde(rename = "Company")]
    company: String,
    #[serde(rename = "Tags")]
    tags: String,
    #[serde(rename = "Themes")]
    themes: String,
    #[serde(rename = "HasBio")]
    has_bio: bool,
    #[serde(rename = "HasSessions")]
    has_sessions: bool,
    #[serde(rename = "IsOfficial")]
    is_official: bool,
    #[serde(rename = "IsPartner")]
    is_partner: bool,
    #[serde(rename = "IsTopSpeaker")]
    is_top_speaker: bool,
    #[serde(rename = "CommunicationManager")]
    communication_manager: String,
    #[serde(rename = "ImageSmallURL")]
    image_small_url: String,
    #[serde(rename = "ImageThumbnailURL")]
    image_thumbnail_url: String,
    #[serde(rename = "ImageLargeURL")]
    image_large_url: String,
    #[serde(rename = "ImageMainURL")]
    image_main_url: String,
}

fn fetch_page_content(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .context("Failed to build HTTP client")?;

    log::info!("Fetching content from URL: {url}");

    let response = client
        .get(url)
        .send()
        .context("Failed to send HTTP request")?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("Server returned non-success status code: {}", status);
    }

    let content = response
        .text()
        .context("Failed to read response body as text")?;

    log::info!("Successfully fetched {} bytes of content", content.len());
    Ok(content)
}

// Unescape Unicode sequences like \u0026 to actual characters
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

// Extract JSON data from HTML - looks for escaped JSON array pattern
fn extract_json_from_html(html_content: &str) -> Result<String> {
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
                        return Ok(unescape_unicode(&unescaped));
                    }
                }
                _ => {}
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not find speaker data JSON in the HTML content"
    ))
}

// Save HTML for debugging if extraction fails
fn save_debug_html(html_content: &str, filename: &str) -> Result<()> {
    let mut file = File::create(filename).context("Failed to create debug HTML file")?;

    file.write_all(html_content.as_bytes())
        .context("Failed to write HTML content to debug file")?;

    log::info!("Saved debug HTML to: {filename}");
    println!("💾 Debug HTML saved to: {filename}");
    Ok(())
}

fn parse_speakers_from_json(json_str: &str) -> Result<Vec<Speaker>> {
    let speakers: Vec<Speaker> =
        serde_json::from_str(json_str).context("Failed to parse JSON data into Speaker structs")?;

    log::info!("Successfully parsed {} speakers from JSON", speakers.len());
    Ok(speakers)
}

// Convert Speaker structs to CSV-ready format
fn convert_to_csv_records(speakers: Vec<Speaker>) -> Vec<SpeakerRecord> {
    speakers
        .into_iter()
        .map(|speaker| {
            let (image_small, image_thumbnail, image_large, image_main) =
                speaker.image.as_ref().map_or_else(
                    || {
                        (
                            "N/A".to_string(),
                            "N/A".to_string(),
                            "N/A".to_string(),
                            "N/A".to_string(),
                        )
                    },
                    |img| (img.s.clone(), img.t.clone(), img.l.clone(), img.u.clone()),
                );

            SpeakerRecord {
                id: speaker.id,
                first_name: speaker.firstname,
                last_name: speaker.lastname,
                email: speaker.email,
                job_title: speaker.job_title,
                company: speaker.company,
                tags: speaker.tags.join(", "),
                themes: speaker.themes.join(", "),
                has_bio: speaker.has_bio,
                has_sessions: speaker.has_sessions,
                is_official: speaker.is_official,
                is_partner: speaker.is_partner,
                is_top_speaker: speaker.top,
                communication_manager: speaker
                    .communication_manager
                    .unwrap_or_else(|| "N/A".to_string()),
                image_small_url: image_small,
                image_thumbnail_url: image_thumbnail,
                image_large_url: image_large,
                image_main_url: image_main,
            }
        })
        .collect()
}

fn write_records_to_csv(records: &[SpeakerRecord], output_path: &Path) -> Result<()> {
    let file = File::create(output_path)
        .with_context(|| format!("Failed to create CSV file at: {}", output_path.display()))?;

    let mut writer = csv::Writer::from_writer(file);

    for record in records {
        writer
            .serialize(record)
            .context("Failed to write record to CSV")?;
    }

    writer.flush().context("Failed to flush CSV writer")?;

    log::info!(
        "Successfully wrote {} records to CSV file: {}",
        records.len(),
        output_path.display()
    );
    Ok(())
}

// Main scraper logic for speakers
fn run_scraper(url: &str, output_path: &Path) -> Result<()> {
    println!("🌐 Fetching webpage content...");
    let html_content = fetch_page_content(url)?;

    println!("🔍 Extracting speaker data from HTML...");
    let json_str = match extract_json_from_html(&html_content) {
        Ok(json) => json,
        Err(e) => {
            save_debug_html(&html_content, "debug_vivatech_page.html")?;
            return Err(e);
        }
    };

    println!("📊 Parsing JSON data...");
    let speakers = parse_speakers_from_json(&json_str)?;
    println!("✅ Found {} speakers", speakers.len());

    let records = convert_to_csv_records(speakers);

    println!("💾 Writing data to CSV file...");
    write_records_to_csv(&records, output_path)?;

    println!(
        "✨ Successfully saved speaker data to: {}",
        output_path.display()
    );
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .init();

    println!("🦀 VivaTech Scraper");
    println!("━━━━━━━━━━━━━━━━━━━");

    match cli.target {
        ScrapeTarget::Speakers => {
            println!("🎤 Scraping speakers...");
            let url = cli.url.as_deref().unwrap_or(TARGET_URL);
            let output_file = cli
                .output
                .unwrap_or_else(|| DEFAULT_OUTPUT_FILE.to_string());
            let output_path = Path::new(&output_file);
            run_scraper(url, output_path)?;
        }
        ScrapeTarget::Partners => {
            println!("🤝 Scraping partners...");
            let url = cli.url.as_deref().unwrap_or(partners::PARTNERS_URL);
            let output_file = cli
                .output
                .unwrap_or_else(|| partners::DEFAULT_PARTNERS_OUTPUT.to_string());
            let output_path = Path::new(&output_file);
            run_partners_scraper(url, output_path)?;
        }
    }

    Ok(())
}

// Partners scraper wrapper
fn run_partners_scraper(url: &str, output_path: &Path) -> Result<()> {
    println!("🌐 Fetching webpage content...");
    let html_content = fetch_page_content(url)?;

    println!("🔍 Extracting partner data from HTML...");
    let partners = partners::extract_partners_from_html(&html_content)?;
    println!("✅ Found {} partners", partners.len());

    let records = partners::convert_to_partner_records(partners);

    println!("💾 Writing data to CSV file...");
    partners::write_partners_to_csv(&records, output_path)?;

    println!(
        "✨ Successfully saved partner data to: {}",
        output_path.display()
    );
    Ok(())
}
