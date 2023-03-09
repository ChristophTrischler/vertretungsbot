use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

use serenity::model::id::UserId;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{error, info};
use sqlx::{Row};
use chrono::{ Utc};


use crate::vertretung::vertretungsdings::{Plan, get_day, check_change, ChangeOption};
use crate::DBConnection;


#[command]
async fn get_example(ctx: &Context, msg: &Message, mut _args: Args)->CommandResult{
   
    if let Err(why) = msg.channel_id.send_files(
        ctx, 
        vec!["https://cdn.discordapp.com/attachments/1076531149989494874/1083049384910000259/example-plan.json"], 
        |m| m.content("Hier is the example.json:")
    ).await{
        error!("faild sending example {why}");
    }
    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message, mut _args: Args)->CommandResult{
    Ok(())
}