use crate::controllers::handlers::api_calls::Job;

pub fn job_to_text(job: &Job) -> String {
    format!(
        "{}. {}. {}. {}",
        job.title,
        job.organisation,
        job.location,
        job.description
    )
}