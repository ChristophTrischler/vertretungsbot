use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

use serenity::model::id::UserId;
use serenity::prelude::*;
use tracing::{error, info};
use sqlx::{Row};
use chrono::{Utc};


use crate::vertretung::vertretungsdings::{Plan, get_day, check_change, ChangeOption};


use crate::DBConnection;    

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