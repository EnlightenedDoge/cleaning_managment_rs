pub mod reader;
pub mod sender;

use std::{collections::HashMap, io::Write, process::exit, thread};

use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use colored::Colorize;
use reader::table::get_soldiers_table;
use sender::send_to;
use std::sync::mpsc;
use table_configs::{config, paths};
use table_maker::Soldier;

const MESSAGE: &str = "***REMOVED*** ***REMOVED***\n***REMOVED*** ***REMOVED*** ***REMOVED*** ***REMOVED***/ה ***REMOVED*** ***REMOVED*** ***REMOVED***";

pub fn start_interface() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_config();

    if !std::path::Path::new(&table_configs::paths::get_output_path(
        &config.output_file_name,
    ))
    .exists()
    {
        println!("Table was not created. Type --help to see how to create one.");
        exit(0);
    }

    let thread_config = config.clone();
    let table = get_soldiers_table(&paths::get_output_path(&config.output_file_name))?;
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
        print!("> ");
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
        } else if input.contains("show") {
            let params:Vec<&str> = input.split_whitespace().collect();
            if params.len()>=2{
                if let Ok(val) = params[1].parse::<usize>(){
                    tx_request_from_main.send(Request::Show(val+1))?;
                } else{
                    println!("Error: Could not parse NUMBER in `show NUMBER`. NUMBER must be a positive integer.");
                }
            }
            else{
                tx_request_from_main.send(Request::Show(2))?;
            }
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
        } else if input.contains("drop") {
            let count = input.split_whitespace().count();
            let mut params = input.split_whitespace();
            if count == 3 {
                params.next();
                let action = params.next().unwrap();
                let date = params.next().unwrap();
                if let Ok(date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                    match action {
                        "postpone" => {
                            tx_request_from_main.send(Request::Drop(DropType::Postpone, date))?
                        }
                        "collapse" => {
                            tx_request_from_main.send(Request::Drop(DropType::Collapse, date))?
                        }
                        "clean" => {
                            tx_request_from_main.send(Request::Drop(DropType::Clean, date))?
                        }
                        _ => println!(
                            "Second parameter must be \"postpone\", \"collapse\" or \"clean\". "
                        ),
                    }
                } else {
                    println!("Date format must be YYYY-MM-DD");
                }
            } else {
                println!("Incorrect number of parameters");
            }
        } else if input.contains("help") {
            println!(
                "Options:
status                                      - Prints current status.
show NUMBER                                 - Show current and NUMBER of following weeks.
switch YYYY-mm-dd YYYY-mm-dd                - Switch between two given dates and update the original table.
drop [clean|collapse|postpone] YYYY-mm-dd   - Remove a date. 
                                                Clean    - Simply remove the date.
                                                Collapse - Replace given date's name with the next date's one. 
                                                           repeat for every following date.
                                                Postpone - Move given date's name one day forward and repeat
                                                           for every following name.
resend                                      - Send the message again disregarding built-in limitation.
help                                        - Display this text."
            )
        }
        input.clear();
    }
}

fn action_loop(
    transmitting: mpsc::Sender<Status>,
    receiving: mpsc::Receiver<Request>,
    config: &config::Config,
    soldiers_table: HashMap<NaiveDate, Soldier>,
) {
    let send_time = config.send_time;
    let reset_time = config.reset_time;
    let output_path = &format!("{}", paths::get_output_path(&config.output_file_name));
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
                        print_around_date(&soldiers_table, 5, &vec![date1, date2]);
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
                        drop_name(&mut soldiers_table, drop_type, date, config);
                        reader::table::update_soldiers_table(&output_path, &soldiers_table)
                            .unwrap();
                        print_around_date(&soldiers_table, 5, &vec![date]);
                    }
                }
                Request::Show(num_of_weeks) => {
                    println!("");
                    let mut now = chrono::Local::now().naive_local().date();
                    if now.weekday() != Weekday::Sun {
                        let prev = now.checked_sub_signed(chrono::Duration::days(6)).unwrap();
                        let prev_sun = prev
                            .iter_days()
                            .filter(|d| d.weekday() == Weekday::Sun)
                            .next()
                            .unwrap();
                        now = prev_sun;
                    }
                    let weeks = now.iter_weeks().take(num_of_weeks);
                    for week in weeks {
                        week.iter_days().take(7).for_each(|day| {
                            if soldiers_table.contains_key(&day) {
                                println!(
                                    "{} {} | {}",
                                    day.weekday().to_string(),
                                    day,
                                    soldiers_table.get(&day).unwrap().name
                                );
                            }
                        });
                        println!("---------------------");
                    }
                    print!("> ");
                    std::io::stdout().flush().unwrap();
                }
            }
        }
    }
}

fn drop_name(
    soldiers_table: &mut HashMap<NaiveDate, Soldier>,
    drop_type: DropType,
    date: NaiveDate,
    config: &config::Config,
) -> HashMap<NaiveDate, Soldier> {
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
            let mut curr_date = iter.next();
            //Iterate over dates. Put the next date's value into the current one. Delete last one.
            while let Some(date) = curr_date {
                let next_date = iter.next();
                match next_date {
                    Some(next_date) => {
                        let next_value = soldiers_table.get(&next_date).unwrap().clone();
                        soldiers_table.entry(date).and_modify(|e| *e = next_value);
                    }
                    None => {
                        soldiers_table.remove(&date);
                    }
                }
                curr_date = next_date;
            }
        }
        DropType::Postpone => {
            //Add "Next eligible date" functionality to table maker.
            //Move modifying functionality to table_maker.
            let mut table = soldiers_table.clone();
            let latest = table.keys().max().unwrap();
            if let Ok(excluded_dates) = reader::table::get_excluded_dates() {
                let mut dates: Vec<NaiveDate> =
                    table.keys().filter(|d| **d >= date).cloned().collect();
                //find next date that isn't a weekend and isn't in the excluded days section
                let mut next_date = latest.iter_days().filter(|x: &NaiveDate| {
                    !config.weekend.contains(&x.weekday())
                        && excluded_dates.iter().filter(|p| p.date == *x).count() == 0
                });
                _ = next_date.next();
                dates.push(next_date.next().unwrap());
                dates.sort();
                let mut iter = dates.into_iter().rev();
                let mut curr_date = iter.next();
                //Iterate over dates. Put the next date's value into the current one. Delete last one.
                while let Some(date) = curr_date {
                    let prev_date = iter.next();
                    match prev_date {
                        Some(prv) => {
                            let _a = date.to_string();
                            let _b = prv.to_string();
                            let _i = table.contains_key(&date);
                            let prev_value = table.get(&prv).unwrap().clone();
                            table.insert(date, prev_value);
                        }
                        None => {
                            table.remove(&date);
                        }
                    }
                    curr_date = prev_date;
                }
            }
            *soldiers_table = table;
        }
    }
    soldiers_table.clone()
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

fn print_around_date(table: &HashMap<NaiveDate, Soldier>, range: usize, dates: &Vec<NaiveDate>) {
    if dates.is_empty() || table.is_empty() {
        return;
    }
    print!("");
    let mut table_dates: Vec<NaiveDate> = table.keys().map(|x| x.clone()).collect();
    table_dates.sort();
    let mut dates_to_print = vec![];
    for date in dates {
        let index;
        let found_index = table_dates.iter().position(|x| *x == *date);
        //If date exists in the keys list unwarp and assign its index.
        if found_index.is_some() {
            index = found_index.unwrap();
        } else {
            //Otherwise find the closest date's index to it.
            if let Some(pos) = table_dates
                .iter()
                .position(|x| x == table_dates.iter().filter(|x| *x < date).last().unwrap())
            {
                index = pos;
            } else {
                index = table_dates
                    .iter()
                    .position(|x| x == table_dates.iter().filter(|x| *x > date).last().unwrap())
                    .unwrap();
            }
        };

        let start_index = index.checked_sub(range).get_or_insert(0).clone();
        let end_index = index + range;
        let end_index = if end_index > table_dates.len() {
            table_dates.len() - 1
        } else {
            end_index
        };

        for date in &table_dates[start_index..end_index] {
            dates_to_print.push(date);
        }
    }
    dedup(&mut dates_to_print);
    for date in dates_to_print {
        if dates.contains(date) {
            println!(
                "{}",
                format!("{} | {}", date, table.get(date).unwrap().name)
                    .red()
                    .bold()
            );
        } else {
            println!("{} | {}", date, table.get(date).unwrap().name);
        }
    }
    print!("> ");
    std::io::stdout().flush().unwrap();
}

fn dedup<T>(v: &mut Vec<T>)
where
    T: Eq + std::hash::Hash + Copy,
{
    let mut uniques = std::collections::HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

enum Request {
    Status,
    Refresh,
    Switch(NaiveDate, NaiveDate),
    Resend,
    Drop(DropType, NaiveDate),
    Show(usize),
}

struct Status {
    sent_today: bool,
    status: String,
    todays_soldier: Option<Soldier>,
    tomorrows_soldier: Option<Soldier>,
}

#[derive(Debug)]
enum DropType {
    Clean,
    Collapse,
    Postpone,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn drop_clean() {
        let data = inititate(DropType::Clean);
        let mut name_table = data.name_table;
        let drop_date = data.drop_date;
        let config = data.config;
        let following_date = name_table
            .keys()
            .filter(|x| **x > drop_date)
            .min()
            .unwrap()
            .clone();
        let following_name = name_table.get(&following_date).unwrap().clone();
        drop_name(&mut name_table, DropType::Clean, drop_date, &config);
        assert!(name_table.get(&drop_date).is_none());
        assert_eq!(
            name_table.get(&following_date).unwrap().name,
            following_name.name
        );
    }
    #[test]
    fn drop_post() {
        let mut data = inititate(DropType::Collapse);
        let mut name_table = data.name_table;
        drop_name(
            &mut name_table,
            DropType::Postpone,
            data.drop_date,
            &data.config,
        );
        println!("{:?}", &name_table);
        assert!(name_table.contains_key(&NaiveDate::from_ymd(***REMOVED***, 5, 22)));
    }

    #[test]
    fn drop_collapse() {
        let data = inititate(DropType::Collapse);
        let mut name_table = data.name_table;
        let org_table = name_table.clone();
        let drop_date = data.drop_date;
        let config = data.config;

        let following_date = name_table
            .keys()
            .filter(|x| **x > drop_date)
            .min()
            .unwrap()
            .clone();
        let following_name = name_table.get(&following_date).unwrap().clone();

        let last_date = name_table.keys().max().unwrap().clone();

        drop_name(&mut name_table, DropType::Collapse, drop_date, &config);

        assert!(name_table.get(&drop_date).unwrap().name == following_name.name);
        assert!(!name_table.keys().any(|x| *x == last_date));
        assert!(
            name_table
                .keys()
                .filter(|x| config.weekend.contains(&x.weekday()))
                .count()
                == 0
        );
        let mut res = name_table
            .iter()
            .filter(|x| *x.0 >= drop_date)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(NaiveDate, Soldier)>>();
        res.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

        let mut org = org_table
            .iter()
            .filter(|x| *x.0 >= drop_date)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(NaiveDate, Soldier)>>();
        org.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        let mut org_iter = org.iter();
        _ = org_iter.next();
        let zipped = org_iter.zip(res.iter());
        for (org, res) in zipped {
            assert_eq!(org.1.name, res.1.name);
        }
    }

    fn inititate(drop_type: DropType) -> Data {
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
        let table_path = format!("./test_table_{:?}.csv", drop_type);
        std::fs::write(&table_path, table).unwrap();
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
        let config_path = format!("./test_config_{:?}.csv", drop_type);

        std::fs::write(&config_path, config).unwrap();
        let table = reader::table::get_soldiers_table(&table_path).unwrap();
        let config = reader::config::load_config(&config_path).unwrap();
        std::fs::remove_file(&table_path).unwrap();
        std::fs::remove_file(&config_path).unwrap();
        Data {
            drop_date: NaiveDate::from_ymd(***REMOVED***, 4, 26),
            name_table: table,
            config,
        }
    }
    struct Data {
        drop_date: NaiveDate,
        name_table: HashMap<NaiveDate, Soldier>,
        config: ConfigReader,
    }
}
