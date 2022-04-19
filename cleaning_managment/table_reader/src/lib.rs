pub mod reader;
pub mod sender;

use std::{thread, time::Duration, collections::HashMap};

use reader::{config::*,table::get_soldiers_table};
use sender::send_to;
use chrono::{NaiveTime, NaiveDate};
use table_maker::Soldier;
use std::sync::mpsc;

pub fn start()->Result<(),Box<dyn std::error::Error>>{
    let config = load_config()?;
    let table = get_soldiers_table(&config.output_path);
    
    Ok(())
}

fn send_loop(transmiting:mpsc::Sender<Status>,send_time:&NaiveTime,soldiers_table:HashMap<NaiveDate,Soldier>){
    while true{
        let now = chrono::Local::now();
        if now.time()>*send_time{
            match soldiers_table.get(&now.date().naive_local()) {
                Some(soldier) => {},
                None => {},
            }
        }
        thread::sleep(Duration::from_secs(***REMOVED***));
    }
}

struct Status<'a>{
    sent_today:bool,
    todays_soldier:Option<&'a Soldier>,
    tommorows_soldier:Option<&'a Soldier>,
}