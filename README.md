# VivaTech Conference Data Scraper 🚀

A Rust-based web scraper that extracts speaker and partner information from the VivaTech conference website into CSV files for better visit planning.

## 🎯 Purpose

This tool scrapes **publicly available data** from the VivaTech conference website and converts it into easy-to-use CSV files. I originally created this for my personal AI planner agent to help optimize my VivaTech visit, and I'm open-sourcing it so other agents and attendees can benefit too!

## 📥 Direct Download Links

Get the latest scraped data directly:

- **[Download Speakers CSV](https://raw.githubusercontent.com/hghalebi/vivatech-scraper/main/vivatech_speakers_2025_extended.csv)** (725+ speakers)
- **[Download Partners CSV](https://raw.githubusercontent.com/hghalebi/vivatech-scraper/main/vivatech_partners_2025.csv)** (2,884 companies)

*Last updated: January 2025*

## 📊 What It Extracts

### Speakers Data (727+ speakers)
- Full names and job titles
- Companies and roles
- Session availability
- Biography availability
- Tags and themes
- Speaker photos URLs
- Official/Partner/Top speaker status

### Partners Data (2,800+ companies)
- Company names
- Categories (startup, partner)
- Countries of origin
- Company descriptions
- Website URLs
- Logo URLs

## 🤔 Why Use This?

Planning your VivaTech visit can be overwhelming with thousands of speakers and exhibitors. With this data in CSV format, you can:

- **Filter speakers** by company, topics, or speaker type
- **Find startups** from specific countries or sectors
- **Plan your route** by identifying must-see exhibitors
- **Import into your calendar** or planning tools
- **Feed to AI assistants** for personalized recommendations
- **Create networking lists** based on your interests

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/hghalebi/vivatech-scraper.git
cd vivatech-scraper

# Build the project
cargo build --release

# Scrape speaker data
cargo run --release -- speakers

# Scrape partner/exhibitor data
cargo run --release -- partners

# Or specify custom output files
cargo run --release -- speakers -o my_speakers.csv
cargo run --release -- partners -o my_partners.csv
```

## 📁 Output Files

- `vivatech_speakers_2025_extended.csv` - All speaker information
- `vivatech_partners_2025.csv` - All partner/exhibitor information

## 🛠️ Requirements

- Rust 1.70 or higher
- Internet connection

## 📋 CSV Format

### Speakers CSV Columns
```
ID, FirstName, LastName, Email, JobTitle, Company, Tags, Themes, 
HasBio, HasSessions, IsOfficial, IsPartner, IsTopSpeaker, 
CommunicationManager, ImageSmallURL, ImageThumbnailURL, 
ImageLargeURL, ImageMainURL
```

### Partners CSV Columns
```
CompanyName, Category, Country, Description, Website, LogoURL
```

## 🤖 For AI Agents & Developers

This data is perfect for:
- **Recommendation engines** - Match attendees with relevant speakers/companies
- **Route optimization** - Plan efficient paths through the exhibition
- **Networking tools** - Identify key contacts based on interests
- **Analytics** - Understand conference trends and focus areas
- **Chatbots** - Answer questions about speakers and exhibitors

Example use cases:
```python
# Find all AI-focused speakers
ai_speakers = df[df['Tags'].str.contains('Artificial Intelligence', na=False)]

# Find all French startups
french_startups = partners_df[
    (partners_df['Category'] == 'startup') & 
    (partners_df['Country'] == 'France')
]
```

## 📊 Data Statistics

- **727** speakers from **600+** companies
- **2,884** partners including **2,500+** startups
- Companies from **50+** countries
- Wide range of sectors: AI, FinTech, HealthTech, CleanTech, and more

## ⚖️ Legal & Ethical Use

- This tool only accesses **publicly available information** from the VivaTech website
- The data is the same as what any visitor can see on the conference website
- Please use responsibly and respect VivaTech's terms of service
- Consider caching data locally to avoid excessive requests

## 🤝 Contributing

Feel free to submit issues, fork the repository, and create pull requests. Some ideas for contributions:
- Add more data fields
- Export to different formats (JSON, Excel)
- Add data analysis features
- Create visualization tools

## 👨‍💻 Author

Created with ❤️ by [Hamze Ghalebi](https://www.linkedin.com/in/hamze/)

- **LinkedIn**: [https://www.linkedin.com/in/hamze/](https://www.linkedin.com/in/hamze/)
- **Twitter/X**: [@Hamzeml](https://x.com/Hamzeml)
- **GitHub**: [@hghalebi](https://github.com/hghalebi)

Feel free to connect if you're interested in AI, social impact, or conference planning tools!

## 🙏 Acknowledgments

Thanks to VivaTech for organizing such an amazing conference and making speaker/partner information publicly accessible for attendees to plan their visits!

---

**Note:** This is an unofficial tool created by an attendee for attendees. It's not affiliated with or endorsed by VivaTech.

Happy conference planning! 🎉 