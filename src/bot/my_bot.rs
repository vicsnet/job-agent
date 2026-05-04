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
// use crate::controllers::handlers::open_ai::generate_supporting_statement;

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
                let cv_text = user.cv_text.clone().unwrap_or_default();

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
                                &cv_text,
                                cv_embedding,
                                &pool,
                                &telegram_id.as_str(),
                                &client
                            ).await.map_err(|e| e.to_string());
                        
                            //  println!("supporting statement: {}", result.as_ref().unwrap().message.as_ref().unwrap_or(&"No message".to_string()));
                            send_job_results(&bot, msg.chat.id, &result).await?;
                        } else {
                            println!("No CV embedding found for user {}, prompting for CV update...", telegram_id);
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
                                &cv_text,
                                cv_embedding,
                                &pool,
                                &telegram_id.as_str(),
                                &client
                            ).await.map_err(|e| e.to_string());
                            //  println!("supporting statement: {}", result.as_ref().unwrap().message.as_ref().unwrap_or(&"No message".to_string()));

                    send_job_results(&bot, msg.chat.id, &result).await?;
                                // let perssonalised_statement = generate_supporting_statement(cv_text, &job.description, client);
                }

                if let Some(cv_text) = user.cv_text {
                    if !cv_text.is_empty() {
                        bot.send_message(
                            msg.chat.id,
                            "🔍 Finding jobs based on your CV...."
                        ).await?;

                        if let Some(cv_embedding) = user.cv_embedding {
                            // println!("my cv3 {}", user.cv_text.as_ref().unwrap_or(&"No CV text".to_string()));
                            
                              let result = match_cv_to_jobs(
                                &cv_text,
                                cv_embedding,
                                &pool,
                                telegram_id.as_str(),
                                &client
                            ).await.map_err(|e| e.to_string());
                            // println!("supporting statement: {}", result.as_ref().unwrap().message.as_ref().unwrap_or(&"No message".to_string()));
                            send_job_results(&bot, msg.chat.id, &result).await?;
                        }
                    }
                }
            }
            Ok(())
        }
    }).await;
}


// async fn send_job_results(
//     bot: &Bot,
//     chat_id: ChatId,
//     result: &Result<crate::controllers::handlers::job_matching::MatchResponse, String>
// ) -> Result<(), teloxide::RequestError> {
//     match result {
//         Ok(response) => {
//             if response.jobs.is_empty() {
//                 bot.send_message(
//                     chat_id,
//                     "No matching jobs found. Try updating your CV or check back later!"
//                 ).await?;
//             } else {
              
//                 let mut reply = String::from("Here are some job matches for you:\n\n\n");
//                 for (score, job) in response.jobs.iter() {
//                     reply.push_str(&format!("• {} ({:.2})\n{}\n\n\n", job.title, score, job.link));
//                 }
//                 if let Some(message) = &response.message {
//                     println!("{}", message);
//                     reply.push_str(&format!("Personalised Statement:\n{}\n", message));
//                 }
//                 // reply.push_str(&format!("\n\n{}", response.message.as_ref().unwrap_or(&"No message".to_string())));

//                 //   println!("{}", response.message.as_ref().unwrap_or(&"No message".to_string()));
//                 bot.send_message(chat_id, reply).await?;
//             }
//         }
//         Err(_) => {
//             bot.send_message(
//                 chat_id,
//                 "❌ An error occurred while processing your CV. Please try again later."
//             ).await?;
//         }
//     }

//     Ok(())
// }


async fn send_job_results(
    bot: &Bot,
    chat_id: ChatId,
    result: &Result<crate::controllers::handlers::job_matching::MatchResponse, String>,
) -> Result<(), teloxide::RequestError> {
    match result {
        Ok(response) => {
            // No jobs case
            if response.jobs.is_empty() {
                bot.send_message(
                    chat_id,
                    "No matching jobs found. Try updating your CV or check back later!",
                )
                .await?;
                return Ok(());
            }

            const MAX_LEN: usize = 4000;

            // Build messages safely (chunking)
            let mut messages: Vec<String> = Vec::new();
            let mut current = String::from("Here are some job matches for you:\n\n");

            for (score, job) in &response.jobs {
                let entry = format!(
                    "• {} ({:.2})\n{}\n\n",
                    job.title,
                    score,
                    job.link
                );

                // If adding this exceeds Telegram limit, push current chunk
                if current.len() + entry.len() > MAX_LEN {
                    messages.push(current);
                    current = String::new();
                }

                current.push_str(&entry);
            }

            if !current.is_empty() {
                messages.push(current);
            }

            // Send job messages
            for msg in messages {
                bot.send_message(chat_id, msg).await?;
            }

            // Send personalised statement separately (safer + cleaner UX)
            if let Some(statement) = &response.message {
               

                bot.send_message(chat_id, "📝 Personalised Supporting Statement:")
                    .await?;

                // Chunk statement too (LLMs can be long)
                let mut start = 0;
                let chars: Vec<char> = statement.chars().collect();

                while start < chars.len() {
                    let end = (start + MAX_LEN).min(chars.len());
                    let chunk: String = chars[start..end].iter().collect();

                    bot.send_message(chat_id, chunk).await?;
                    start = end;
                }
            }
        }

        Err(e) => {
            eprintln!("Error in send_job_results: {}", e);

            bot.send_message(
                chat_id,
                "❌ An error occurred while processing your CV. Please try again later.",
            )
            .await?;
        }
    }

    Ok(())
}