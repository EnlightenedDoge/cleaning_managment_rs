mod reader;
mod sender;
mod cli;

use std::{collections::HashMap, process::exit, thread, fmt::Display};

use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use colored::Colorize;
use reader::table::get_people_table;
use sender::send_to;
use std::sync::mpsc;
use table_configs::{config, paths};
use table_maker::Person;

const MESSAGE: &str = "תזכורת ניקיון\nבמקרה בו אינך יכול/ה לנקות הודיעו לאחראים";

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
    let table = get_people_table(&paths::get_output_path(&config.output_file_name))?;
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

    cli::start(tx_request_from_main, rx_status).unwrap();
Ok(())
}

//Function responsible for executing relevent code depending on the Request enum
fn action_loop(
    transmitting: mpsc::Sender<Vec<Box<dyn Display + Send>>>,
    receiving: mpsc::Receiver<Request>,
    config: &config::Config,
    people_table: HashMap<NaiveDate, Person>,
) {
    let send_time = config.send_time;
    let reset_time = config.reset_time;
    let output_path = &paths::get_output_path(&config.output_file_name);
    let maintainer = &config.maintainer;
    let alert_day = config.alert_day;
    let mut people_table = people_table;
    let mut is_sent = false;
    let mut status = String::new();
    let mut resend = false;

    //Wait for thread to send a request
    loop {
        if let Ok(req) = receiving.recv() {
            let output:Vec<Box<dyn Display + Send>> = match req {
                //Send back formatted status of current and next candidate
                Request::Status => {
                    let status =
                    Status {
                            sent_today: is_sent,
                            status: status.to_string(),
                            todays_name: get_name_from_table(&people_table, 0),
                            tomorrows_name: get_name_from_table(&people_table, 1),
                        };
                   vec![Box::new(format!("sent_today: {}
sent status: {}
today's candidate: {:?}
tomorrow's candidate: {:?}
now: {},
send time: {}
reset time: {}",
                                        status.sent_today,
                                        status.status,
                                        status.todays_name,
                                        status.tomorrows_name,
                                        chrono::Local::now(),
                                        config.send_time,
                                        config.send_time > config.reset_time))]
                            
                }

                //basic functionality. Send to specified name on specified time
                Request::Refresh => {
                    (is_sent, status) = check_can_send(
                        &people_table,
                        &send_time,
                        &reset_time,
                        &alert_day,
                        &maintainer,
                        is_sent,
                        status,
                        resend,
                    );
                    resend = false;
                    continue;
                }

                //switch names of between two dates
                Request::Switch(date1, date2) => {
                    if people_table.contains_key(&date1) && people_table.contains_key(&date2) {
                        let sol = people_table.get(&date1).unwrap().clone();
                        people_table.insert(date1, people_table.get(&date2).unwrap().clone());
                        people_table.insert(date2, sol);
                        reader::table::update_source_table(&output_path, &people_table)
                            .unwrap();
                        print_around_date(&people_table, 5, &vec![date1, date2])
                    } else {
                        vec![Box::new("Dates provided don't exist in table")]
                    }
                }

                //toggle flag for sending a message again
                Request::Resend => {
                    resend = true;
                    vec![]
                }

                //Drop a name from the table completly, collapse the next names to the current one's date, or postpone by 
                //moving all names from given date one entry forward
                Request::Drop(drop_type, date) => {
                    if people_table.contains_key(&date) {
                        drop_name(&mut people_table, drop_type, date, config);
                        reader::table::update_source_table(&output_path, &people_table)
                            .unwrap();
                        print_around_date(&people_table, 5, &vec![date])
                    }
                    else{
                        vec![]
                    }
                }

                //Show x+1 weeks from, and including, current week.
                Request::Show(num_of_weeks) => {
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
                    let mut output = Vec::<Box<dyn Display+Send>>::new();
                    let weeks = now.iter_weeks().take(num_of_weeks);
                    for week in weeks {
                        week.iter_days().take(7).for_each(|day| {
                            if people_table.contains_key(&day) {
                                output.push(Box::new(format!(
                                    "{} {} | {}",
                                    day.weekday().to_string(),
                                    day,
                                    people_table.get(&day).unwrap().name
                                )));
                            }
                        });
                        output.push(Box::new("---------------------"));
                    }
                    output
                }
            };
            transmitting.send(output).unwrap();
        }
    }
}

fn drop_name(
    people_table: &mut HashMap<NaiveDate, Person>,
    drop_type: DropType,
    date: NaiveDate,
    config: &config::Config,
) {
    match drop_type {
        DropType::Clean => _ = people_table.remove(&date),
        DropType::Collapse => {
            //Get dates from and including given point and sort them.
            let mut dates: Vec<NaiveDate> = people_table
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
                        let next_value = people_table.get(&next_date).unwrap().clone();
                        people_table.entry(date).and_modify(|e| *e = next_value);
                    }
                    None => {
                        people_table.remove(&date);
                    }
                }
                curr_date = next_date;
            }
        }
        DropType::Postpone => {
            //Add "Next eligible date" functionality to table maker.
            //Move modifying functionality to table_maker.
            let mut table = people_table.clone();
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
            *people_table = table;
        }
    }
}

//Check if it is possible to send an SMS message and return status.
fn check_can_send(
    people_table: &HashMap<NaiveDate, Person>,
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
        (is_sent, status) = send_from_table(&people_table);

        //send to maintainer
        if !is_sent && chrono::Local::now().date().weekday() == *alert_day {
            send_to(maintainer, "Maintainer alert").unwrap();
            is_sent = true;
        }
    }
    (is_sent, status)
}

//send sms message to number found in table
fn send_from_table(people_table: &HashMap<NaiveDate, Person>) -> (bool, String) {
    match get_name_from_table(&people_table, 0) {
        Some(person) => {
            if let Ok(res) = send_to(&person.phone, &format!("{}: {}", person.name, MESSAGE)) {
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
        None => (false, "No person found".to_string()),
    }
}

//get name-number pair from the table
fn get_name_from_table(
    people_table: &HashMap<NaiveDate, Person>,
    add_days: i64,
) -> Option<Person> {
    people_table
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

//print range of entries around given date
fn print_around_date(table: &HashMap<NaiveDate, Person>, range: usize, dates: &Vec<NaiveDate>) ->Vec<Box<dyn Display+Send>>{
    if dates.is_empty() || table.is_empty() {
        return vec![];
    }
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
    let mut output = Vec::<Box<dyn std::fmt::Display+Send>>::new();
    for date in dates_to_print {
        if dates.contains(date) {
            output.push(Box::new(format!("{} | {}", date, table.get(date).unwrap().name)
                    .red()
                    .bold()));
        } else {
            output.push(Box::new(format!("{} | {}", date, table.get(date).unwrap().name)));
        }
    }
    output
}

//Helper function to deduplicate a collection
fn dedup<T>(v: &mut Vec<T>)
where
    T: Eq + std::hash::Hash + Copy,
{
    let mut uniques = std::collections::HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

//Types of actions to execute in action_loop
pub enum Request {
    Status,
    Refresh,
    Switch(NaiveDate, NaiveDate),
    Resend,
    Drop(DropType, NaiveDate),
    Show(usize),
}

pub struct Status {
    sent_today: bool,
    status: String,
    todays_name: Option<Person>,
    tomorrows_name: Option<Person>,
}

#[derive(Debug)]
pub enum DropType {
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
        assert!(name_table.contains_key(&NaiveDate::from_ymd(2022, 5, 22)));
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
            .collect::<Vec<(NaiveDate, Person)>>();
        res.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

        let mut org = org_table
            .iter()
            .filter(|x| *x.0 >= drop_date)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(NaiveDate, Person)>>();
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
John,9725130465,2022-05-18
Maddy,972541235467,2022-05-17
Kaladin,972468578448, 2022-05-16";
        let table_path = format!("./test_table_{:?}.csv", drop_type);
        std::fs::write(&table_path, table).unwrap();
        let config = r#"{
    "start_date": "2022-05-18",
    "range": 180,
    "output_path":"./output/",
    "send_time":"09:00:00",
    "reset_time":"01:00:00",
    "maintainer":"",
    "alert_day":"5",
    "weekend":[5,6,7]
    }"#;
        let config_path = format!("./test_config_{:?}.csv", drop_type);

        std::fs::write(&config_path, config).unwrap();
        let table = reader::table::get_people_table(&table_path).unwrap();
        let config = reader::config::load_config(&config_path).unwrap();
        std::fs::remove_file(&table_path).unwrap();
        std::fs::remove_file(&config_path).unwrap();
        Data {
            drop_date: NaiveDate::from_ymd(2022, 5, 18),
            name_table: table,
            config,
        }
    }
    struct Data {
        drop_date: NaiveDate,
        name_table: HashMap<NaiveDate, Person>,
        config: ConfigReader,
    }
}
