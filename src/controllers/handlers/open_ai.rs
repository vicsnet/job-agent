use reqwest::Client;

pub async fn generate_supporting_statement(
    cv_text: &str,
    job_description: &str,
    // person_spec: &str,
    client: &Client
) -> Result<String, Box<dyn std::error::Error>> {
let prompt = format!(
r#"
You are an expert UK NHS job application writer with a strong understanding of NHS shortlisting and TRAC applications.

Your task is to write a highly tailored supporting statement based on the CV and job description provided.

CRITICAL INSTRUCTIONS:
- Use British English
- Write in a professional, natural, and confident tone
- Do NOT use generic or vague phrases
- Do NOT repeat the CV verbatim
- Do NOT include bullet points or headings
- Write in full paragraphs

STRUCTURE:
1. Start with a strong opening paragraph stating:
   - The role being applied for
   - A concise summary of experience and suitability

2. Main body:
   - Systematically address the PERSON SPECIFICATION criteria
   - Map skills and experience directly to the job requirements
   - Use specific, evidence-based examples from the CV
   - Where possible, apply STAR (Situation, Task, Action, Result) implicitly (not labelled)

3. NHS VALUES:
   - Reflect NHS values such as patient-centred care, teamwork, respect, and continuous improvement
   - Show real examples, not just statements

4. Closing paragraph:
   - Reinforce suitability
   - Show enthusiasm for the role and organisation

STYLE REQUIREMENTS:
- ATS-friendly language
- Clear, concise, and impactful
- Avoid repetition
- Avoid clichés like "hardworking" or "team player" without evidence

OUTPUT:
- A strong, compelling supporting statement suitable for NHS job applications
- Ideally between 1100–1400 words unless specified otherwise

JOB DESCRIPTION:
{}

CV:
{}

Now write the supporting statement.
"#,
job_description,
cv_text
);

    let response = client
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(std::env::var("OPEN_API_KEY")?)
        .json(&serde_json::json!({
        "model": "gpt-4.1-mini",
        "input": prompt
    }))
        .send().await?;

    let body: serde_json::Value = response.json().await?;
    let text = body["output"][0]["content"][0]["text"].as_str().unwrap_or("").to_string();

    Ok(text)
}


#[cfg(test)]
mod tests{
    use super::*;
    use dotenvy::dotenv;
    #[tokio::test]
    async fn test_generate_supporting_statement() {
        let client = Client::new();
        dotenv().ok();
        let cv_text = "John Doe has 5 years of experience in software development, specializing in Rust and web applications. He has a proven track record of delivering high-quality code and collaborating effectively with cross-functional teams.";
        let job_description = "We are looking for a software developer with experience in Rust to join our team. The ideal candidate will have a strong background in web application development and be able to work well in a collaborative environment.";

        let result = generate_supporting_statement(cv_text, job_description, &client).await.unwrap();
        println!("Generated Supporting Statement:\n{}", result);
    }
}