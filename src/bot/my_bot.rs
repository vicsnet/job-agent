use teloxide::{ prelude::*, types::User };
use dotenvy::dotenv;
use crate::controllers::handlers::{
    job_matching::match_cv_to_jobs,
    users::get_user_by_telegram_id,
};

use sqlx::PgPool;
use reqwest::Client;
use crate::controllers::handlers::users::{ create_user, update_user_cv };

pub async fn run_bot(pool: PgPool, client: Client) {
    dotenv().ok();

    let token = std::env::var("BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let bot = Bot::new(token);

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let pool = pool.clone();
        let client = client.clone();

        async move {
            if let Some(text) = msg.text() {

                let telegram_id = msg.chat.id.to_string();

                create_user(&pool, &msg.chat.id.to_string()).await.unwrap_or_else(|e| {
                    eprintln!("Error creating user: {}", e);
                });
                
                if text == "/start" {

                    bot.send_message(msg.chat.id, "👋 Send me your resume").await?;
                    return respond(());
                }

                let existing_user = get_user_by_telegram_id(
                    &pool,
                    &msg.chat.id.to_string()
                ).await.unwrap_or_else(|e| {
                    eprintln!("Error fetching user: {}", e);
                    None
                });

                if existing_user.is_none() {
                    bot.send_message(msg.chat.id, "💾 Saving your CV...").await?;

                    update_user_cv(
                        &pool,
                        &msg.chat.id.to_string(),
                        text,
                        &client
                    ).await.unwrap_or_else(|e| {
                        eprintln!("Error updating user CV: {}", e);
                    });
                    bot.send_message(msg.chat.id, "💾 Saving your CV...").await?;
                    bot.send_message(msg.chat.id, "⏳ Processing your CV...").await?;
                } else {
                    bot.send_message(msg.chat.id, "⏳ Processing your CV...").await?;
                }

                let result = match_cv_to_jobs(text, &pool, &client).await;

                match result {
                    Ok(response) => {
                        if response.jobs.is_empty() {
                            bot.send_message(
                                msg.chat.id,
                                "No matching jobs found. Try updating your CV or check back later!"
                            ).await?;
                        } else {
                            let mut reply = String::from("Here are some job matches for you:\n\n");
                            for (score, job) in response.jobs.iter() {
                                reply.push_str(
                                    &format!("• {} ({:.2})\n{}\n\n", job.title, score, job.link)
                                );
                            }
                            bot.send_message(msg.chat.id, reply).await?;
                        }
                    }
                    Err(_) => {
                        bot.send_message(
                            msg.chat.id,
                            "❌ An error occurred while processing your CV. Please try again later."
                        ).await?;
                    }
                }
            }

            respond(())
        }
    }).await;
}
