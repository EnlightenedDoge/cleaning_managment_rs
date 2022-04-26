pub mod reader;
pub mod sender;

use std::{collections::HashMap, thread};

use chrono::{NaiveDate, NaiveTime};
use reader::{config::*, table::get_soldiers_table};
use sender::send_to;
use std::sync::mpsc;
use table_maker::Soldier;

const MESSAGE: &str = "***REMOVED*** ***REMOVED***\n***REMOVED*** ***REMOVED*** ***REMOVED*** ***REMOVED***/ה ***REMOVED*** ***REMOVED*** ***REMOVED***";

pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let table = get_soldiers_table(&config.output_path)?;
    let (tx_request_from_main, rx_request) = mpsc::channel();
    let (tx_status, rx_status) = mpsc::channel();
    let rx_request_clock = tx_request_from_main.clone();

    let _logic_thread = thread::spawn(move || {
        send_loop(
            tx_status,
            rx_request,
            &config.send_time,
            &config.reset_time,
            table,
        )
    });
    let _clock_thread = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(60 * 5));
        rx_request_clock.send(Request::Refresh).unwrap();
    });


    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.contains("status")  {
                tx_request_from_main.send(Request::Status)?;
                println!("fetching data:");
                let status = rx_status.recv()?;
                println!("sent_today: {}
                sent status: {}
                today's soldier: {:?}
                tommorow's soldier: {:?}
                now: {},
                send time: {}
                reset time: {}",status.sent_today,status.status, status.todays_soldier, status.tommorows_soldier,chrono::Local::now(),config.send_time,config.send_time>config.reset_time)
            }
        }
    Ok(())
}

fn send_loop(
    transmiting: mpsc::Sender<Status>,
    receiving: mpsc::Receiver<Request>,
    send_time: &NaiveTime,
    reset_time: &NaiveTime,
    soldiers_table: HashMap<NaiveDate, Soldier>,
) {
    let mut is_sent = false;
    let mut status = 0;
    loop {
        if let Ok(req) = receiving.recv() {
            //refresh variables
            let now = chrono::Local::now();
            if !is_sent && now.time() > *send_time && now.time() < *reset_time {
                match get_soldier(&soldiers_table, 0) {
                    Some(soldier) => {
                        // if let Ok(res) = send_to(&soldier.phone, MESSAGE) {
                        //     let num: u32 = res.parse().unwrap();
                        //     if num > 0 {
                        //         is_sent = true;
                        //         status = num;
                        //     } else {
                        //         is_sent = false;
                        //         status = num;
                        //     }
                        // }
                        is_sent=true;
                        status=69;
                    }
                    None => {}
                }
            } else {
                is_sent = false;
                status = 0;
            }
            match req {
                Request::Status => {
                    transmiting
                        .send(Status {
                            sent_today: is_sent,
                            status,
                            todays_soldier: get_soldier(&soldiers_table, 0),
                            tommorows_soldier: get_soldier(&soldiers_table, 1),
                        })
                        .unwrap();
                }
                Request::Refresh => {}
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

enum Request {
    Status,
    Refresh,
}

struct Status {
    sent_today: bool,
    status: u32,
    todays_soldier: Option<Soldier>,
    tommorows_soldier: Option<Soldier>,
}
