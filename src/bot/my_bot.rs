use teloxide::{ prelude::*, types::User };
use dotenvy::dotenv;
use crate::controllers::handlers::{ job_matching::match_cv_to_jobs };
use crate::controllers::embedding::text_to_vec::get_embeddings;

use sqlx::PgPool;
use reqwest::Client;
use crate::controllers::handlers::users::{
    create_user,
    update_user_cv,
    update_user_state,
    get_user_by_telegram_id,
};

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
                    println!("Error creating user: {}", e);
                });

                let existing_user = get_user_by_telegram_id(&pool, &telegram_id).await.unwrap();
                let user = existing_user.unwrap();

                if text == "/start" {
                    let has_cv = user.cv_text
                        .as_deref()
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                    if has_cv {
                        bot.send_message(
                            msg.chat.id,
                            "🔍 Finding jobs based on your existing CV..."
                        ).await?;

                        if let Some(cv_embedding) = user.cv_embedding {
                            let result = match_cv_to_jobs(
                                cv_embedding,
                                &pool,
                                &client
                            ).await.map_err(|e| e.to_string());
                            send_job_results(&bot, msg.chat.id, result).await?;
                        } else {
                            bot.send_message(
                                msg.chat.id,
                                "⚠️ No CV embedding found. Please send /update_cv to re-upload your CV."
                            ).await?;
                        }
                    } else {
                        update_user_state(&pool, &telegram_id, "awaiting_cv").await.unwrap_or_else(
                            |e| {
                                println!("Error updating user state: {}", e);
                            }
                        );
                        bot.send_message(msg.chat.id, "👋 Send me your resume").await?;
                    }

                    return Ok(());
                }

                if text == "/update_cv" {
                    update_user_state(&pool, &telegram_id, "awaiting_cv").await.unwrap_or_else(|e| {
                        eprintln!("Error updating user state: {}", e);
                    });
                    bot.send_message(
                        msg.chat.id,
                        "📝 Please send me your new CV and I'll update it."
                    ).await?;
                    return Ok(());
                }

                if user.state == "awaiting_cv" {
                    bot.send_message(msg.chat.id, "💾 Saving your CV...").await?;
                    update_user_cv(
                        &pool,
                        &msg.chat.id.to_string(),
                        &text,
                        &client
                    ).await.unwrap_or_else(|e| {
                        eprintln!("Error updating user CV: {}", e);
                    });

                    update_user_state(&pool, &telegram_id, "idle").await.unwrap_or_else(|e| {
                        eprintln!("Error updating user state: {}", e);
                    });

                    let cv_embedding = get_embeddings(text, &client).await.unwrap();

                    bot.send_message(msg.chat.id, "✅ CV saved! Finding jobs...").await?;

                   

                    let result = match_cv_to_jobs(
                                cv_embedding,
                                &pool,
                                &client
                            ).await.map_err(|e| e.to_string());

                    send_job_results(&bot, msg.chat.id, result).await?;
                }

                if let Some(cv_text) = user.cv_text {
                    if !cv_text.is_empty() {
                        bot.send_message(
                            msg.chat.id,
                            "🔍 Finding jobs based on your CV...."
                        ).await?;

                        if let Some(cv_embedding) = user.cv_embedding {
                            
                              let result = match_cv_to_jobs(
                                cv_embedding,
                                &pool,
                                &client
                            ).await.map_err(|e| e.to_string());
                            send_job_results(&bot, msg.chat.id, result).await?;
                        }
                    }
                }
            }
            Ok(())
        }
    }).await;
}

async fn send_job_results(
    bot: &Bot,
    chat_id: ChatId,
    result: Result<crate::controllers::handlers::job_matching::MatchResponse, String>
) -> Result<(), teloxide::RequestError> {
    match result {
        Ok(response) => {
            if response.jobs.is_empty() {
                bot.send_message(
                    chat_id,
                    "No matching jobs found. Try updating your CV or check back later!"
                ).await?;
            } else {
                let mut reply = String::from("Here are some job matches for you:\n\n");
                for (score, job) in response.jobs.iter() {
                    reply.push_str(&format!("• {} ({:.2})\n{}\n\n", job.title, score, job.link));
                }
                bot.send_message(chat_id, reply).await?;
            }
        }
        Err(_) => {
            bot.send_message(
                chat_id,
                "❌ An error occurred while processing your CV. Please try again later."
            ).await?;
        }
    }

    Ok(())
}
