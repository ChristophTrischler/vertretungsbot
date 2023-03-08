
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{error, info};
use sqlx::{Row};
use chrono::{ Utc};


use crate::vertretung::vertretungsdings::{Plan, get_v_text, get_day, ChangeOption};


use crate::DBConnection;    

#[command]
pub async fn update(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult{
    let id = msg.author.id.0 as i64;
    info!("{} used !update", id);
    
    let connection = {
        let data_read = ctx.data.read().await;
        data_read.get::<DBConnection>().unwrap().clone()
    };

   
    let query = 
    sqlx::query("SELECT \"embed\", \"data\" FROM \"user\" WHERE \"discord_id\" = $1")
    .bind(id);
    let row = query.fetch_one(connection.as_ref()).await.expect("faild query");
    
    let embed_activated: bool = row.try_get(0).unwrap();
    let data: String = row.try_get(1).unwrap();
    let plan: Plan = serde_json::from_str(&data).unwrap();


    let mut date = (Utc::now() - chrono::Duration::days(1)).naive_utc().date();
    let mut vdays = Vec::new();
    for i in 1..=5{
        match get_v_text(i, &mut date).await{
            ChangeOption::Some(vday) => vdays.push(vday),
            ChangeOption::Same => continue,
            ChangeOption::None => break,
        }
        if vdays.len() >= 3 {
            break;
        }
    }
    for vday in vdays{
        let day = get_day(&vday, &plan);
        
        if let Err(why) = msg.channel_id.send_message(ctx, |m| {
            if embed_activated {
                day.to_embed(m);
                m
            }
            else {
                m.content(day.to_string())
            }
        }).await {
            error!("Error sending Message: {:?}", why);
        }
    }
    Ok(())
}