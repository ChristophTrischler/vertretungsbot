use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

use serenity::model::id::UserId;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{error, info};
use reqwest::Client;
use sqlx::{Row};
use chrono::{ Utc};


use crate::commands::send_plan::vertretungsdings::{Plan, get_v_text, get_day, check_change, ChangeOption, VDay};

mod vertretungsdings;

use crate::DBConnection;


#[command]
pub async fn send_plan(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult{
    let id = msg.author.id.0 as i64;
    info!("{} used !send_plan", id);  

    let connection = {
        let data_read = ctx.data.read().await;
        data_read.get::<DBConnection>().unwrap().clone()
    };  
    

    let opt_url = msg.attachments.first();
    if opt_url.is_none() {
        send_file_error(ctx, msg).await;
        return Ok(());
    }
    let url = &opt_url.unwrap().url;

    let client = Client::new();
    let opt_plan: Option<Plan> = client.get(url)
    .send().await
    .unwrap()
    .json().await 
    .ok();


    if opt_plan.is_none() {
        send_file_error(ctx, msg).await;
        return Ok(());
    }
    let plan = opt_plan.unwrap();

    let plan_str = serde_json::to_string(&plan).unwrap();

    sqlx::query("INSERT INTO \"user\" VALUES ($1,$2,$3,$4) 
        ON CONFLICT (discord_id) DO UPDATE SET \"data\" = EXCLUDED.data")
        .bind(id)
        .bind(true)
        .bind(false)
        .bind(plan_str)
        .execute(connection.as_ref())
        .await?;

    Ok(())
}

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
    let mut vdays: Vec<VDay> = Vec::new();
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

#[command]
pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult{
    let id = msg.author.id.0 as i64;
    info!("{} used !activate", id);
    let status = args.single::<bool>()?;
    

    let connection = {
        let data_read = ctx.data.read().await;
        data_read.get::<DBConnection>().unwrap().clone()
    };

    sqlx::query("UPDATE \"user\" SET \"active\"=$1 WHERE \"discord_id\"=$2")
    .bind(status)   
    .bind(id)
    .execute(connection.as_ref())
    .await?;

    Ok(())
}

async fn send_file_error(ctx: &Context, msg: &Message){
    if let Err(why) = msg.channel_id.say(&ctx.http, "Error with atteched file").await {
        error!("Error sending message: {:?}", why);
    }
}

#[command]
pub async fn embed(ctx: &Context, msg: &Message, mut args: Args)->CommandResult{
    let id = msg.author.id.0 as i64;
    info!("{} used !activate", id);
    let arg = args.single::<bool>()?;

    let connection = {
        let data_read = ctx.data.read().await;
        data_read.get::<DBConnection>().unwrap().clone()
    };

    sqlx::query("UPDATE \"user\" SET \"embed\"=$1 WHERE \"discord_id\"=$2")
    .bind(arg)
    .bind(id)
    .execute(connection.as_ref())
    .await?;

    Ok(())
}


pub async fn check_loop(arc_ctx: Arc<Context>){
    let min15 = Duration::from_secs(900);
    let mut times = HashMap::new();
    loop {
        let mut vdays = Vec::new();
        let mut date = (Utc::now() - chrono::Duration::days(1)).naive_utc().date();
        for i in 1..=5{
            let last = if let Some(s) = times.get_mut(&i) {
                s
            } else {
                times.insert(i, String::new());
                times.get_mut(&i).unwrap() 
            };
            
            
            match check_change(i, last, &mut date).await{
                ChangeOption::Some(vday) => vdays.push(vday),
                ChangeOption::Same => continue,
                ChangeOption::None => break,
            };
            
            if vdays.len() >= 3 {
                break;
            }
        }


        if vdays.is_empty(){
            continue;
        }

        let ctx: &Context = arc_ctx.as_ref();
        let connection = {
            let data_read = ctx.data.read().await;
            data_read.get::<DBConnection>().unwrap().clone()
        };

        let query = sqlx::query(
            "SELECT \"discord_id\", \"embed\", \"data\" FROM \"user\" WHERE \"active\" = true"
        ); 
        let rows = query.fetch_all(connection.as_ref())
        .await
        .unwrap();

        for row in rows {
            let id: i64 = row.try_get(0).unwrap();
            let embed_activated: bool = row.try_get(1).unwrap();
            let data = row.try_get(2).unwrap();

            let user = UserId(id as u64)
            .to_user(ctx)
            .await
            .unwrap();
            
            let plan: Plan = serde_json::from_str(data).unwrap();

            for vday in &vdays {
                let day = get_day(vday, &plan); 


                if let Err(why) = user.direct_message(ctx,|m|{
                    if embed_activated {
                        day.to_embed(m);
                        m
                    }
                    else {
                        m.content(day.to_string())
                    }
                }).await {
                    error!("Error sending dm: {:?}", why);
                }
            }
        }
        
        
        info!("cheked for updates");

        tokio::time::sleep(min15).await;
    }

}