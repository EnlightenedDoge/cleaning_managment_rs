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
    let config = load_config(reader::CONFIG_PATH)?;
    let thread_config = config.clone();
    let table = get_soldiers_table(&format!("{}output_table.csv", config.output_path))?;
    let (tx_request_from_main, rx_request) = mpsc::channel();
    let (tx_status, rx_status) = mpsc::channel();
    let rx_request_clock = tx_request_from_main.clone();

    //run the thread responsible for reading data and sending messages
    let _logic_thread =
        thread::spawn(move || action_loop(tx_status, rx_request, &thread_config, table));
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
    config: &ConfigReader,
    soldiers_table: HashMap<NaiveDate, Soldier>,
) {
    let send_time = config.send_time;
    let reset_time = config.reset_time;
    let output_path = &config.output_path;
    let maintainer = &config.maintainer;
    let alert_day = config.alert_day;
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
                    (is_sent, status) = check_can_send(
                        &soldiers_table,
                        &send_time,
                        &reset_time,
                        &alert_day,
                        &maintainer,
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
                        reader::table::update_soldiers_table(&output_path, &soldiers_table)
                            .unwrap();
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
                        drop_name(drop_type, date, &mut soldiers_table, config);
                    }
                }
            }
        }
    }
}

fn drop_name(
    drop_type: DropType,
    date: NaiveDate,
    soldiers_table: &mut HashMap<NaiveDate, Soldier>,
    config: &ConfigReader,
) {
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
                    let next_value = soldiers_table.get(&next_date).unwrap().clone();
                    soldiers_table.entry(date).and_modify(|e| *e = next_value);
                } else {
                    soldiers_table.remove(&date);
                }
            }
        }
        DropType::Postpone => {
            //Add "Next eligible date" functionality to table maker.
            //Move modifying functionality to table_maker.
            let latest = soldiers_table.keys().max().unwrap();
            if let Ok(excluded_dates) = reader::table::get_excluded_dates(&config) {
                let mut dates: Vec<NaiveDate> = soldiers_table
                    .keys()
                    .filter(|d| **d >= date)
                    .cloned()
                    .collect();
                //find next date that isn't a weekend and isn't in the excluded days section
                let next_date = latest
                    .iter_days()
                    .filter(|x: &NaiveDate| {
                        !config.weekend.contains(&x.weekday())
                            && excluded_dates.iter().filter(|p| p.date == *x).count() == 0
                    })
                    .next()
                    .unwrap();
                dates.push(next_date);
                dates.sort();

                let mut iter = dates.into_iter();

                //Iterate over dates. Put the next date's value into the current one. Delete last one.
                while let Some(date) = iter.next_back() {
                    if let Some(prev_date) = iter.next_back() {
                        let prev_value = soldiers_table.get(&prev_date).unwrap().clone();
                        soldiers_table.entry(date).and_modify(|e| *e = prev_value);
                    } else {
                        soldiers_table.remove(&date);
                    }
                }
            }
        }
    }
}

fn check_can_send(
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
        if !is_sent && chrono::Local::now().date().weekday() == *alert_day {
            send_to(maintainer, "Maintainer alert").unwrap();
            is_sent = true;
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
                    (true, res)
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
    Postpone,
}

#[cfg(test)]
mod tests{
    use std::time::Instant;

    use super::*;

    #[test]
    fn drop_clean(){
        let mut data = inititate();
        let following_date = data.name_table.keys().filter(|x|**x>data.drop_date).min().unwrap().clone();
        let following_name = data.name_table.get(&following_date).unwrap().clone();
        drop_name(DropType::Clean, data.drop_date, &mut data.name_table, &data.config);
        assert!(data.name_table.get(&data.drop_date).is_none());
        assert_eq!(data.name_table.get(&following_date).unwrap().name,following_name.name);
    }

    struct Data{
        drop_date: NaiveDate,
        name_table: HashMap<NaiveDate, Soldier>,
        config: ConfigReader,
    }

    fn inititate()->Data{
        let table = "name,number,date
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-04-12
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-04-13
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-04-24
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-04-25
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-04-26
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-04-27
***REMOVED*** ***REMOVED***-***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-01
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-02
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-03
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-04
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-08
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-09
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-10
***REMOVED***  ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-11
***REMOVED*** ***REMOVED***-***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-15
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-16
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-17
***REMOVED*** ***REMOVED***,***REMOVED***-***REMOVED***,***REMOVED***-05-18";
        std::fs::write("./test_table.csv", table).unwrap();
        let config = r#"{
	"start_date": "***REMOVED***-04-12",
	"range": ***REMOVED***,
	"output_path":"./output/",
	"send_time":"09:00:00",
	"reset_time":"01:00:00",
	"maintainer":"***REMOVED***",
	"alert_day":"5",
	"weekend":[5,6,7]
}"#;
        std::fs::write("./test_config.csv", config).unwrap();
        let table = reader::table::get_soldiers_table("./test_table.csv").unwrap();
        let config = reader::config::load_config("./test_config.csv").unwrap();
        std::fs::remove_file("./test_table.csv").unwrap();
        std::fs::remove_file("./test_config.csv").unwrap();
        Data { drop_date: NaiveDate::from_ymd(***REMOVED***,4,26), name_table: table, config: config }
    }
}