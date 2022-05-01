pub mod reader;
pub mod sender;

use std::{collections::HashMap, io::Write, thread};

use chrono::{NaiveDate, NaiveTime};
use reader::{config::*, table::get_soldiers_table};
use sender::send_to;
use std::sync::mpsc;
use table_maker::Soldier;

const MESSAGE: &str = "***REMOVED*** ***REMOVED***\n***REMOVED*** ***REMOVED*** ***REMOVED*** ***REMOVED***/ה ***REMOVED*** ***REMOVED*** ***REMOVED***";

pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let table = get_soldiers_table(&format!("{}output_table.csv",config.output_path))?;
    let (tx_request_from_main, rx_request) = mpsc::channel();
    let (tx_status, rx_status) = mpsc::channel();
    let rx_request_clock = tx_request_from_main.clone();

    let _logic_thread = thread::spawn(move || {
        send_loop(
            tx_status,
            rx_request,
            &config.send_time,
            &config.reset_time,
            &config.output_path,
            table,
        )
    });
    let _clock_thread = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(10));
        rx_request_clock.send(Request::Refresh).unwrap();
    });

    loop {
        print!(">");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.contains("status") {
            tx_request_from_main.send(Request::Status)?;
            println!("fetching data:");
            let status = rx_status.recv()?;
            println!(
                "sent_today: {}
sent status: {}
today's soldier: {:?}
tommorow's soldier: {:?}
now: {},
send time: {}
reset time: {}",
                status.sent_today,
                status.status,
                status.todays_soldier,
                status.tommorows_soldier,
                chrono::Local::now(),
                config.send_time,
                config.send_time > config.reset_time
            )
        } else if input.contains("switch") {
            let count = input.split_whitespace().count();
            let mut params = input.split_whitespace();
            if count != 3 {
                println!("Incorrect number of parameters");
            } else {
                params.next();
                if let Ok(date1) = NaiveDate::parse_from_str(params.next().unwrap(), "%Y-%m-%d") {
                    if let Ok(date2) = NaiveDate::parse_from_str(params.next().unwrap(), "%Y-%m-%d")
                    {
                        tx_request_from_main.send(Request::Switch(date1, date2))?;
                    } else {
                        println!("Second date could not be parsed");
                    }
                } else {
                    println!("First date could not be parsed");
                }
            }
        }
        input.clear();
    }
}

fn send_loop(
    transmiting: mpsc::Sender<Status>,
    receiving: mpsc::Receiver<Request>,
    send_time: &NaiveTime,
    reset_time: &NaiveTime,
    output_path: &str,
    soldiers_table: HashMap<NaiveDate, Soldier>,
) {
    let mut soldiers_table = soldiers_table;
    let mut is_sent = false;
    let mut status = String::new();
    loop {
        if let Ok(req) = receiving.recv() {
            //refresh variables
            if is_close_to_time(reset_time) {
                is_sent = false;
                status.clear();
            }
            if !is_sent && is_close_to_time(send_time) {
                match get_soldier(&soldiers_table, 0) {
                    Some(soldier) => {
                        if let Ok(res) =
                            send_to(&soldier.phone, &format!("{}: {}", soldier.name, MESSAGE))
                        {
                            //DEBUG
                            send_to("***REMOVED***", &format!("{}: {}", soldier.name, MESSAGE)).unwrap();
                            let num: u32 = res.split_whitespace().filter(|s|s.parse::<u32>().is_ok()).next().get_or_insert("0").parse().unwrap();
                            if num > 0 {
                                is_sent = true;
                                status = res;
                            } else {
                                is_sent = false;
                                status = res;
                            }
                        }
                        else{
                        status="Failed".to_string();
                        is_sent=false;
                        }
                    }
                    None => {
                        status="No soldier found".to_string();
                    }
                }
            } 
            match req {
                Request::Status => {
                    transmiting
                        .send(Status {
                            sent_today: is_sent,
                            status:status.to_string(),
                            todays_soldier: get_soldier(&soldiers_table, 0),
                            tommorows_soldier: get_soldier(&soldiers_table, 1),
                        })
                        .unwrap();
                }
                Request::Refresh => {}
                Request::Switch(date1, date2) => {
                    if soldiers_table.contains_key(&date1) && soldiers_table.contains_key(&date2) {
                        let sol = soldiers_table.get(&date1).unwrap().clone();
                        soldiers_table.insert(date1, soldiers_table.get(&date2).unwrap().clone());
                        soldiers_table.insert(date2, sol);
                        reader::table::update_soldiers_table(output_path, &soldiers_table).unwrap();
                    } else {
                        println!("Dates provided don't exist in table");
                    }
                }
            }
        }
    }
}

fn get_soldier(soldiers_table: &HashMap<NaiveDate, Soldier>, add_days: i64) -> Option<Soldier> {
    soldiers_table
        .get(
            &chrono::Local::now()
                .date()
                .naive_local()
                .checked_add_signed(chrono::Duration::days(add_days))
                .unwrap(),
        )
        .cloned()
}

fn is_close_to_time(time:&NaiveTime)->bool{
    (NaiveTime::from(chrono::Local::now().time()) - *time).num_minutes().abs()<=1
}

enum Request {
    Status,
    Refresh,
    Switch(NaiveDate, NaiveDate),
}

struct Status {
    sent_today: bool,
    status: String,
    todays_soldier: Option<Soldier>,
    tommorows_soldier: Option<Soldier>,
}
