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


use crate::vertretung::vertretungsdings::{Plan, get_day, check_change, ChangeOption};
use crate::DBConnection;


#[command]
async fn get_example()->CommandResult{
    Ok(())
}

#[command]
async fn help()->CommandResult{
    Ok(())
}