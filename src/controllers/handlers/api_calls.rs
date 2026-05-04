use reqwest::Client;
use serde::{ Deserialize, Serialize};

use scraper::{ Html, Selector, ElementRef };
use chrono::{ NaiveDate, DateTime, Utc, TimeZone };
use tokio::time::{ sleep, Duration };
use sqlx::{ PgPool };
use crate::helpers::job_to_text::job_to_text;
use crate::controllers::embedding::text_to_vec::get_embeddings;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub title: String,
    pub organisation: String,
    pub location: String,
    pub salary: String,
    pub posted_datetime: Option<DateTime<Utc>>,
    pub closing_date: Option<DateTime<Utc>>,
    pub link: String,
    pub description: String,
    pub embedding: Option<Vec<f32>>,
}

// ---------------- DATE PARSER ----------------
pub fn parse_nhs_date(date_str: &str) -> Option<DateTime<Utc>> {

        let cleaned = date_str
        .replace("Closing date:", "")
        .replace("Date posted:", "")
        .replace('\n', " ")
        .replace('\u{00A0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

   

    let naive = NaiveDate::parse_from_str(&cleaned, "%-d %B %Y").ok()?;

    Some(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?))
}

pub async fn save_job(pool: &PgPool, job: &Job) -> Result<(), sqlx::Error> {

    sqlx
        ::query(
            r#"
        INSERT INTO jobs (
            id, title, organisation, location, salary,
            posted_date, closing_date, link, description, embedding
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        ON CONFLICT (id)
        DO UPDATE SET
            description = EXCLUDED.description
        "#
        )
        .bind(&job.id)
        .bind(&job.title)
        .bind(&job.organisation)
        .bind(&job.location)
        .bind(&job.salary)
        .bind(&job.posted_datetime)
        .bind(&job.closing_date)
        .bind(&job.link)
        .bind(&job.description)
        .bind(&job.embedding)
        .execute(pool).await?;

    Ok(())
}

pub async fn delete_expired_jobs(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM jobs WHERE closing_date IS NOT NULL AND closing_date  < NOW()")
        .execute(pool)
        .await?;
    Ok(())
}

// Fetch job Listing from nhs jobs website
pub async fn fetch_jobs(client: &Client, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; JobAgent/1.0)")
        .header("Accept", "text/html")
        .header("Accept-Language", "en-GB,en;q=0.9")
        .send().await?;

    let status = response.status();

    let result = response.text().await?;
    if !status.is_success() {
        return Err(format!("Failed to fetch jobs: HTTP {}", status).into());
    }

    Ok(result)
}

// Extract job id to reduce duplicates job
fn extract_job_id(link: &str) -> String {
    link.split("/jobadvert/").nth(1).unwrap_or("").split("?").next().unwrap_or("").to_string()
}

async fn fetch_job_description(html: &str) -> String {
    let document = Html::parse_document(html);

    let container_selector = Selector::parse("main").unwrap();

    let Some(container) = document.select(&container_selector).next() else {
        return String::new();
    };

    let mut result = String::new();

    for node in container.descendants() {
        if let Some(el) = ElementRef::wrap(node) {
            let tag = el.value().name();

            let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();

            if text.is_empty() {
                continue;
            }

            match tag {
                "h2" => {
                    result.push_str(&format!("\n\n{}\n", text.to_uppercase()));
                }
                "p" => {
                    result.push_str(&format!("{}\n", text));
                }
                "li" => {
                    result.push_str(&format!("• {}\n", text));
                }
                _ => {}
            }
        }
    }

    result.trim().to_string()
}

// async fn fetch_job_person_specification(html: &str) -> String{
//     "hello".to_string()
// }

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
       

        let closing_datetime = parse_nhs_date(&closing_date);


        jobs.push(Job {
            id,
            title,
            organisation,
            location,
            salary,
            posted_datetime,
            closing_date: closing_datetime,
            link,
            description: "".to_string(),
            embedding: None,
        });
    }

    jobs
}

pub async fn fetch_all_jobs(
    keyword: &str,
    client: &Client,
    pool: &PgPool
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
            println!("No more jobs found, stopping.");
            break;
        }

        // descritpion fetching

        let mut filled_descriptions = Vec::new();

        for mut job in jobs {
            let desc_html = fetch_jobs(client, &job.link).await?;

            // getting description
            let description = fetch_job_description(&desc_html).await;
         

            job.description = description;

            let job_id = extract_job_id(&job.link);
            job.id = job_id;

            let job_test = job_to_text(&job);

            let job_description_vec = match get_embeddings(&job_test, client).await {
                Ok(e) => e,
                Err(err) => {
                    println!("❌ Embedding failed for {}: {}", job.id, err);
                    continue; // skip this job
                }
            };
            job.embedding = Some(job_description_vec);

            // println!("Fetched description for job: {}", );
            if let Err(e) = save_job(pool, &job).await {
                println!("❌ DB error for {}: {}", &job.id, e);
            }
            filled_descriptions.push(job);
        }
        // break;
        all_jobs.extend(filled_descriptions);
        page += 1;

        if page > 5 {
            println!("Reached page limit.");
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }
    Ok(all_jobs)
}

pub async fn job_fetch_scheduler(pool: &PgPool, client: &Client) {
   
   let keywords = vec![
    "Nursing & Midwifery",
    "Health Science Services",
    "Support Services",
    "Administrative Services",
    "Medical & Dental",
    "Emergency Services",
    "Allied Health Professions",
    "Personal Social Services",
    "Dirctors",
    "project management",
    "data analysis",
    "healthcare assistant",
   ];

   loop{

    println!("Starting job fetch cycle...");

    delete_expired_jobs(&pool).await.unwrap();

    for keyword in &keywords {
        println!("Fetching jobs for keyword: '{}'", keyword);

        match fetch_all_jobs(keyword, client, pool).await {

            Ok(jobs) => {
                println!("Fetched {} jobs for '{}'", jobs.len(), keyword)
            },
            Err(e) => {
                println!("Error fetching jobs for '{}': {}", keyword, e)
            },
        }

        sleep(Duration::from_secs(60)).await; // Wait 1 minute before next keyword
    }

    println!("😴 Sleeping before next cycle...");

    sleep(Duration::from_secs(60 * 60 * 6)).await;
   }

}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_fetch_jobs() {
        let client = Client::new();
        let pool = PgPool::connect(
            "postgres://job_user:strongpassword@localhost/job_agent"
        ).await.unwrap();

        let jobs = fetch_all_jobs("project management", &client, &pool).await.unwrap();
        assert!(!jobs.is_empty(), "Should fetch some jobs");
        dbg!("Fetched {} jobs", jobs.len());
        dbg!(&jobs);
    }
    #[tokio::test]
    async fn test_delete_expired_jobs() {
        let pool = PgPool::connect(
            "postgres://job_user:strongpassword@localhost/job_agent"
        ).await.unwrap();

        delete_expired_jobs(&pool).await.unwrap();
    }
}
