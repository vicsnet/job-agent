use crate::controllers::handlers::api_calls::Job;
use crate::controllers::embedding::text_to_vec::get_embeddings;
use crate::helpers::similarities::check_similarity;
use crate::controllers::handlers::users::{ get_already_sent_jobs, save_user_job };
use crate::controllers::handlers::open_ai::generate_supporting_statement;

use sqlx::{ PgPool, Row };
use reqwest::Client;

#[derive(Debug)]
pub struct MatchResponse {
    pub jobs: Vec<(f32, Job)>,
    pub message: Option<String>,
}
pub async fn match_cv_to_jobs(
    cv_text: &str,
    cv_embedding: Vec<f32>,
    pool: &PgPool,
    telegram_id: &str,
    client: &Client
) -> Result<MatchResponse, Box<dyn std::error::Error + Send + Sync>> {
 
    
    let all_jobs = sqlx
    ::query(
        "SELECT id, title, description, location, organisation, salary, posted_date, closing_date, link, embedding FROM jobs WHERE embedding IS NOT NULL"
    )
    .fetch_all(pool).await?;

let mut scored_jobs = Vec::new();

for job in all_jobs {
    let job_embedding_f64: Vec<f64> = job.try_get("embedding")?;
    
    let job_embedding: Vec<f32> = job_embedding_f64
    .into_iter()
    .map(|x| x as f32)
    .collect();

let id: &str = job.try_get("id")?;

let already_sent = get_already_sent_jobs(pool, telegram_id, id).await?;

if already_sent {
   
    continue; // Skip jobs that have already been sent to the user
}

let title = job.try_get("title")?;
// let description = job.try_get("description")?;
let location = job.try_get("location")?;
let organisation = job.try_get("organisation")?;
let salary = job.try_get("salary")?;
let posted_datetime = job.try_get("posted_date")?;
let closing_date = job.try_get("closing_date")?;
let link = job.try_get("link")?;
let description = job.try_get("description")?;

let score = check_similarity(&cv_embedding, &job_embedding);
if score > 0.2 {
    scored_jobs.push((
                score,
                Job {
                    id: id.to_string(),
                    title: title,
                    organisation: organisation,
                    location: location,
                    salary: salary,
                    posted_datetime: posted_datetime,
                    closing_date: closing_date,
                    link: link,
                    description: description,
                    embedding: None,
                },
            ));
        }
    }

    scored_jobs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let top_jobs: Vec<(f32, Job)> = scored_jobs.into_iter().take(1).collect();

    if top_jobs.is_empty() {
        Ok(MatchResponse {
            jobs: vec![],
            message: Some("No strong matches found. Try improving your CV or using".to_string()),
        })
    } else {
        let mut personalised_statement = String::new();

        if let Some((_, job)) = top_jobs.first() {
     
            save_user_job(pool, telegram_id, &job.id).await?;
            personalised_statement = generate_supporting_statement(cv_text, &job.description, client).await.unwrap();
        }
        Ok(MatchResponse {
            jobs: top_jobs,
            message: Some(personalised_statement),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_match_cv_to_jobs() {
        let pool = PgPool::connect(
            "postgres://job_user:strongpassword@localhost/job_agent"
        ).await.unwrap();
        let client = Client::new();

        let cv_text =
            "VINCENT ADEBISI ADESANMI
+4407979295249 | Birmingham | Gmail 
PROFESSIONAL SUMMARY
_______________________________________________________________________________________________
Results-oriented Data Analyst with a solid foundation in data management, visualisation, and SQL optimisation. Proven experience utilising Power BI and Excel to transform complex datasets into actionable insights for the construction and agricultural sectors. Currently pursuing an MSc in International Project Management, combining technical analytical skills with strategic project oversight to support evidence-based decision-making and operational efficiency.
CORE COMPETENCIES
_______________________________________________________________________________________________
·	Data Analysis & Visualisation: Power BI, Microsoft Excel (Pivot Tables, Advanced Formulas, Charts).
·	Database Management: SQL (Querying, Optimisation, Extraction), Data Cleaning, Data Validation.
·	Reporting: Performance Monitoring, Resource Allocation, Budget Tracking, Stakeholder Reporting.
·	Soft Skills: Cross-functional Collaboration, Problem Solving, Quality Assurance, Remote Teamwork.
WORK EXPERIENCE
_______________________________________________________________________________________________
Asteryx Construction,  Abuja, Nigeria
Data Analyst                                  05/2024 – 08/2024 
·	Managed SQL databases to track project budgets, timelines, and resources across multiple construction projects.
·	Created and maintained Power BI dashboards to visualise key metrics and support evidence-based decision-making.
·	Produced detailed Excel reports using pivot tables, formulas, and charts to monitor performance and highlight potential risks.
·	Collaborated with cross-functional teams to ensure data accuracy, quality, and alignment with project objectives.
Spring Farm, Ondo State, Nigeria
Data Analyst (Student Intern)                            06/2023 - 02/2024 
·	Collected, validated, and analysed raw project data to track resource allocation and identify expenditure trends.
·	Designed clear Excel dashboards and visualisations tailored for project managers and key stakeholders to track progress.
·	Utilised SQL queries for troubleshooting data issues, extracting relevant datasets, and optimising reporting processes.
·	Supported the implementation of quality assurance checks, ensuring consistent and accurate reporting across the department.
Olooluade Nig Ltd, Ondo State, Nigeria 
Data Assistant                                         01/2020 – 02/2021 
·	Conducted preliminary data analysis to identify operational inefficiencies and highlight specific areas for process improvement.
·	Optimised SQL query performance to significantly improve the speed and efficiency of reporting.
·	Assisted in the collection and cleaning of large datasets to ensure reliability for construction project reporting.
EDUCATION
_______________________________________________________________________________________________
YORK ST JOHN’S UNIVERSITY, LONDON CAMPUS | MSc INTERNATIONAL PROJECT MANAGEMENT                               01/2024 – Present 
FEDERAL UNIVERSITY OF TECHNOLOGY AKURE | B.ENG METALLURGICAL AND MATERIALS ENGINEERING                         11/2015 – 10/2021 
Researchwork:Atmospheric Corrosion Mapping of the Federal University of Technology Akure.
REFERENCE(S) 
Available on request
";
        let cv_embedding = get_embeddings(cv_text, &client).await.unwrap();
        let telegram_id = "1234567890";

        let result = match_cv_to_jobs(cv_text,cv_embedding, &pool, telegram_id, &client).await;
        assert!(result.is_ok());
        let scored_jobs = result.unwrap();
        dbg!("Scored Jobs: {}", scored_jobs);
    }
}
