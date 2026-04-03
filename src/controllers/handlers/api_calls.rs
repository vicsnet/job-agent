use reqwest::Client;
use serde::{ Deserialize, Serialize };

use scraper::{ Html, Selector };
use chrono::{ NaiveDate, DateTime, Utc, TimeZone };
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub title: String,
    pub organisation: String,
    pub location: String,
    pub salary: String,
    pub posted_date_raw: String,
    pub posted_datetime: Option<DateTime<Utc>>,
    pub closing_date: String,
    pub link: String,
}

// ---------------- DATE PARSER ----------------
pub fn parse_nhs_date(date_str: &str) -> Option<DateTime<Utc>> {
    let cleaned = date_str.replace("Date posted:", "").trim().to_string();

    let naive = NaiveDate::parse_from_str(&cleaned, "%-d %B %Y").ok()?;

    Some(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?))
}

// Fetch job Listing from nhs jobs website
pub async fn fetch_jobs(client: &Client, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; JobAgent/1.0)")
        .send().await?;

    let status = response.status();

    let result = response.text().await?;
    if !status.is_success() {
        return Err(format!("Failed to fetch jobs: HTTP {}", status).into());
    }

    Ok(result)
}

// Extract the jobs from the nhs website
pub fn extract_jobs(html: &str) -> Vec<Job> {
    let document = Html::parse_document(html);

    let job_selector = Selector::parse("li[data-test='search-result']").unwrap();
    let title_selector = Selector::parse("a[data-test='search-result-job-title']").unwrap();
    let org_selector = Selector::parse("div[data-test='search-result-location'] h3").unwrap();
    let salary_selector = Selector::parse("li[data-test='search-result-salary']").unwrap();
    let posted_selector = Selector::parse("li[data-test='search-result-publicationDate']").unwrap();
    let closing_selector = Selector::parse("li[data-test='search-result-closingDate']").unwrap();

    let mut jobs = Vec::new();

    for job in document.select(&job_selector) {
        let title_el = match job.select(&title_selector).next() {
            Some(el) => el,
            None => {
                continue;
            }
        };

        let href = title_el.value().attr("href").unwrap_or("");

        let id = href.split('/').last().unwrap_or("").to_string();

        let title = title_el.text().collect::<Vec<_>>().join("").trim().to_string();

        let link = format!("https://www.jobs.nhs.uk{}", href);

        let organisation = job
            .select(&org_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        let location = organisation.clone();

        let salary = job
            .select(&salary_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        let posted_raw = job
            .select(&posted_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        let posted_datetime = parse_nhs_date(&posted_raw);

        let closing_date = job
            .select(&closing_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        jobs.push(Job {
            id,
            title,
            organisation,
            location,
            salary,
            posted_date_raw: posted_raw,
            posted_datetime,
            closing_date,
            link,
        });
    }

    jobs
}

pub async fn fetch_all_jobs(
    keyword: &str,
    client: &Client
) -> Result<Vec<Job>, Box<dyn std::error::Error>> {
    let url = format!("https://www.jobs.nhs.uk/candidate/search/results?keyword={}", keyword);

    let mut all_jobs = Vec::new();
    let mut page = 1;

    loop {
        let paged_url = format!("{}&page={}", url, page);
        println!("Fetching page {}: {}", page, paged_url);

        let html = fetch_jobs(client, &paged_url).await?;
        let jobs = extract_jobs(&html);

        if jobs.is_empty() {
            break;
        }

        all_jobs.extend(jobs);
        page += 1;

        if page > 5 {
            println!("Reached page limit.");
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }
    Ok(all_jobs)
}



#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_fetch_jobs() {
        let client = Client::new();

        let jobs = fetch_all_jobs("nurse", &client).await.unwrap();
        assert!(!jobs.is_empty(), "Should fetch some jobs");
        dbg!("Fetched {} jobs", jobs.len());
        dbg!(&jobs);


    }
}
