pub mod reader;
pub mod sender;

use std::{collections::HashMap, io::Write, thread};

use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use reader::{config::*, table::get_soldiers_table};
use sender::send_to;
use std::sync::mpsc;
use table_maker::Soldier;

const MESSAGE: &str = "***REMOVED*** ***REMOVED***\n***REMOVED*** ***REMOVED*** ***REMOVED*** ***REMOVED***/ה ***REMOVED*** ***REMOVED*** ***REMOVED***";

pub fn start_interface() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let table = get_soldiers_table(&format!("{}output_table.csv", config.output_path))?;
    let (tx_request_from_main, rx_request) = mpsc::channel();
    let (tx_status, rx_status) = mpsc::channel();
    let rx_request_clock = tx_request_from_main.clone();

    //run the thread responsible for reading data and sending messages
    let _logic_thread = thread::spawn(move || {
        action_loop(
            tx_status,
            rx_request,
            &config.send_time,
            &config.reset_time,
            &config.output_path,
            &config.maintainer,
            &config.alert_day,
            table,
        )
    });
    //Run the thread to tick the logic_thread every set period of time
    let _clock_thread = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(10));
        rx_request_clock.send(Request::Refresh).unwrap();
    });

    //main thread
    '_user_interface: loop {
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
tomorrow's soldier: {:?}
now: {},
send time: {}
reset time: {}",
                status.sent_today,
                status.status,
                status.todays_soldier,
                status.tomorrows_soldier,
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
                        println!("Second date could not be parsed. Expecting YYYY-mm-dd");
                    }
                } else {
                    println!("First date could not be parsed. Expecting YYYY-mm-dd");
                }
            }
        } else if input.contains("resend") {
            tx_request_from_main.send(Request::Resend)?;
        } else if input.contains("help") {
            println!(
                "Options:
status                            - Prints current status.
switch YYYY-mm-dd YYYY-mm-dd      - Switch between two given dates and update the original table.
resend                            - Send the message again disregarding built-in limitation.
help                              - Display this text."
            )
        }
        input.clear();
    }
}

fn action_loop(
    transmitting: mpsc::Sender<Status>,
    receiving: mpsc::Receiver<Request>,
    send_time: &NaiveTime,
    reset_time: &NaiveTime,
    output_path: &str,
    maintainer: &str,
    alert_day: &Weekday,
    soldiers_table: HashMap<NaiveDate, Soldier>,
) {
    let mut soldiers_table = soldiers_table;
    let mut is_sent = false;
    let mut status = String::new();
    let mut resend = false;

    //Wait for thread to send a request
    loop {
        if let Ok(req) = receiving.recv() {
            match req {
                //Send back formatted status of current and next
                Request::Status => {
                    transmitting
                        .send(Status {
                            sent_today: is_sent,
                            status: status.to_string(),
                            todays_soldier: get_soldier_from_table(&soldiers_table, 0),
                            tomorrows_soldier: get_soldier_from_table(&soldiers_table, 1),
                        })
                        .unwrap();
                }

                //basic functionality. Send to specified on specified time
                Request::Refresh => {
                    (is_sent, status) = tick(
                        &soldiers_table,
                        send_time,
                        reset_time,
                        alert_day,
                        maintainer,
                        is_sent,
                        status,
                        resend,
                    );
                    resend = false;
                }

                //switch names of between two dates
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

                //toggle flag for sending a message again
                Request::Resend => {
                    resend = true;
                }
                Request::Drop(drop_type, date) => {
                    if soldiers_table.contains_key(&date) {
                        match drop_type {
                            DropType::Clean => _ = soldiers_table.remove(&date),
                            DropType::Collapse => {
                                //Get dates from and including given point and sort them.
                                let mut dates: Vec<NaiveDate> = soldiers_table
                                    .keys()
                                    .filter(|d| **d >= date)
                                    .cloned()
                                    .collect();
                                dates.sort();
                                let mut iter = dates.into_iter();

                                //Iterate over dates. Put the next date's value into the current one. Delete last one.
                                while let Some(date) = iter.next() {
                                    if let Some(next_date) = iter.next() {
                                        let next_value =
                                            soldiers_table.get(&next_date).unwrap().clone();
                                        soldiers_table.entry(date).and_modify(|e| *e = next_value);
                                    } else {
                                        soldiers_table.remove(&date);
                                    }
                                }
                            }
                            DropType::Extend => {
                                //Add "Next eligible date" functionality to table maker.
                                //Move modifying functionality to table_maker.
                                let latest = soldiers_table.keys().max();
                                let mut dates: Vec<NaiveDate> = soldiers_table
                                    .keys()
                                    .filter(|d| **d >= date)
                                    .cloned()
                                    .collect();
                                dates.sort();
                                let mut iter = dates.into_iter();

                                //Iterate over dates. Put the next date's value into the current one. Delete first one.
                                while let Some(date) = iter.next_back() {
                                    if let Some(prev_date) = iter.next_back() {
                                        let prev_value =
                                            soldiers_table.get(&prev_date).unwrap().clone();
                                        soldiers_table.entry(date).and_modify(|e| *e = prev_value);
                                    } else {
                                        soldiers_table.remove(&date);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tick(
    soldiers_table: &HashMap<NaiveDate, Soldier>,
    send_time: &NaiveTime,
    reset_time: &NaiveTime,
    alert_day: &Weekday,
    maintainer: &str,
    is_sent: bool,
    status: String,
    resend: bool,
) -> (bool, String) {
    let mut status = status;
    let mut is_sent = is_sent;
    if is_close_to_time(reset_time) {
        is_sent = false;
        status.clear();
    }
    if !is_sent && is_close_to_time(send_time) || resend {
        (is_sent, status) = send_from_table(&soldiers_table);

        //send to maintainer
        if chrono::Local::now().date().weekday() == *alert_day {
            send_to(maintainer, "Maintainer alert").unwrap();
        }
    }
    (is_sent, status)
}

fn send_from_table(soldiers_table: &HashMap<NaiveDate, Soldier>) -> (bool, String) {
    match get_soldier_from_table(&soldiers_table, 0) {
        Some(soldier) => {
            if let Ok(res) = send_to(&soldier.phone, &format!("{}: {}", soldier.name, MESSAGE)) {
                let num: u32 = res
                    .split_whitespace()
                    .filter(|s| s.parse::<u32>().is_ok())
                    .next()
                    .get_or_insert("0")
                    .parse()
                    .unwrap();
                if num > 0 {
                    (false, res)
                } else {
                    (false, res)
                }
            } else {
                // status = "Failed".to_string();
                // is_sent = false;
                (false, "Failed".to_string())
            }
        }
        None => (false, "No soldier found".to_string()),
    }
}

fn get_soldier_from_table(
    soldiers_table: &HashMap<NaiveDate, Soldier>,
    add_days: i64,
) -> Option<Soldier> {
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

fn is_close_to_time(time: &NaiveTime) -> bool {
    (NaiveTime::from(chrono::Local::now().time()) - *time)
        .num_minutes()
        .abs()
        <= 1
}

enum Request {
    Status,
    Refresh,
    Switch(NaiveDate, NaiveDate),
    Resend,
    Drop(DropType, NaiveDate),
}

struct Status {
    sent_today: bool,
    status: String,
    todays_soldier: Option<Soldier>,
    tomorrows_soldier: Option<Soldier>,
}

enum DropType {
    Clean,
    Collapse,
    Extend,
}
