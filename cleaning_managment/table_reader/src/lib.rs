pub mod reader;
pub mod sender;

use std::{thread, collections::HashMap, time::Duration};

use reader::{config::*,table::get_soldiers_table};
use sender::send_to;
use chrono::{NaiveTime, NaiveDate};
use table_maker::Soldier;
use std::sync::mpsc;

const MESSAGE:&str = "***REMOVED*** ***REMOVED***\n***REMOVED*** ***REMOVED*** ***REMOVED*** ***REMOVED***/ה ***REMOVED*** ***REMOVED*** ***REMOVED***";

pub fn start()->Result<(),Box<dyn std::error::Error>>{
    let config = load_config()?;
    let table = get_soldiers_table(&config.output_path)?;
    let (tx_request,rx_request) = mpsc::channel();
    let (tx_status,rx_status) = mpsc::channel();
    let thread = thread::spawn(move||send_loop(tx_status, rx_request, &config.send_time, &config.reset_time, &table));
    loop{

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input {
            _ => {},
        }
        break;
    }
    thread.join().unwrap();
    Ok(())
}

fn send_loop<'a,'b>(transmiting:mpsc::Sender<&Status<'a>>,receiving:mpsc::Receiver<Request>,send_time:&NaiveTime,reset_time:&NaiveTime,soldiers_table: &'a HashMap<NaiveDate,Soldier>){
    let mut is_sent=false;
    let mut status=0;
    loop{ 
        let now = chrono::Local::now();
        if now.time()>*send_time&&now.time()<*reset_time{
            match get_soldier(&soldiers_table, 0) {
                Some(soldier) => {
                    if let Ok(res) = send_to(&soldier.phone, MESSAGE){
                        let num:u32 = res.parse().unwrap();
                        if num>0{
                            is_sent=true;
                            status=num;
                        }
                        else{
                            is_sent=false;
                            status=num;
                        }
                    }
                },
                None => {},
            }
        }
        else{
            is_sent=false;
            status=0;
        }

        if let Ok(req)=receiving.try_recv(){
            match req {
                Request::Status => {
                    transmiting.send(&Status { sent_today: is_sent, status, todays_soldier:get_soldier(soldiers_table, 0) , tommorows_soldier:get_soldier(soldiers_table, 1)  }).unwrap();
                },
            }
        }
        thread::sleep(Duration::from_secs(***REMOVED***));
    }
}

fn get_soldier(soldiers_table: &HashMap<NaiveDate, Soldier>, add_days: i64) -> Option<&Soldier> {
    soldiers_table.get(&chrono::Local::now().date().naive_local().checked_add_signed(chrono::Duration::days(add_days)).unwrap())
}

enum Request{
    Status,
}

struct Status<'a>{
    sent_today:bool,
    status:u32,
    todays_soldier:Option<&'a Soldier>,
    tommorows_soldier:Option<&'a Soldier>,
}