use std::{sync::mpsc::{Sender, Receiver, SendError}, io::Write, fmt::Display};

use chrono::NaiveDate;

use crate::{Request, DropType};
pub fn start(tx_request_from_main:Sender<Request>,rx_output:Receiver<Vec<Box<dyn Display + Send>>>)->Result<(),Box<dyn std::error::Error>>{
    //main thread
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let params:Vec<&str> = input.trim().split_whitespace().collect();
        if params.len()==0{
            continue;
        }
        let output = match params[0]{
            "status"=>fetch_status(&tx_request_from_main,&rx_output)?,
            "show"=>show(&params[1..], &tx_request_from_main,&rx_output)?,
            "switch"=>switch(&params[1..], &tx_request_from_main, &rx_output)?,
            "resend"=>{tx_request_from_main.send(Request::Resend)?;vec![]},
            "drop"=>drop(&params[1..], &tx_request_from_main, &rx_output)?,
            "help"=>vec![Box::new(get_help()) as Box<dyn Display + Send>],
            _=>continue,
        };
        for line in output{
            println!("{line}");
        }
        
        input.clear();
    }
}

fn fetch_status(tx_request_from_main:&Sender<Request>,rx_output:&Receiver<Vec<Box<dyn Display + Send>>>)->Result<Vec<Box<dyn Display + Send>>,Box<dyn std::error::Error>>{
    tx_request_from_main.send(Request::Status)?;
            println!("fetching data:");
            Ok(rx_output.recv()?)
//             println!(
//                 "sent_today: {}
// sent status: {}
// today's soldier: {:?}
// tomorrow's soldier: {:?}
// now: {},
// send time: {}
// reset time: {}",
//                 status.sent_today,
//                 status.status,
//                 status.todays_soldier,
//                 status.tomorrows_soldier,
//                 chrono::Local::now(),
//                 config.send_time,
//                 config.send_time > config.reset_time
//             );
//             Ok(())
}

fn show(params:&[&str],tx_request_from_main:&Sender<Request>,rx_output:&Receiver<Vec<Box<dyn Display + Send>>>)->Result<Vec<Box<dyn Display + Send>>,Box<dyn std::error::Error>>{
            if params.len()>0{
                if let Ok(val) = params.first().unwrap().parse::<usize>(){
                    tx_request_from_main.send(Request::Show(val+1))?;
                    Ok(rx_output.recv().unwrap())
                } else{
                    Ok(vec![Box::new("Error: Could not parse NUMBER in `show NUMBER`. NUMBER must be a positive integer.".to_string())])
                }
            }
            else{
                tx_request_from_main.send(Request::Show(2))?;
                Ok(rx_output.recv()?)
            }
}

fn switch(params:&[&str],tx_request_from_main:&Sender<Request>,rx_output:&Receiver<Vec<Box<dyn Display + Send>>>)->Result<Vec<Box<dyn Display + Send>>,Box<dyn std::error::Error>>{
            if params.len() != 2 {
                Ok(vec![Box::new("Incorrect number of parameters".to_string())])
            } else {
                if let Ok(date1) = NaiveDate::parse_from_str(params[0], "%Y-%m-%d") {
                    if let Ok(date2) = NaiveDate::parse_from_str(params[1], "%Y-%m-%d")
                    {
                        tx_request_from_main.send(Request::Switch(date1, date2))?;
                        Ok(rx_output.recv()?)
                    } else {
                        Ok(vec![Box::new("Second date could not be parsed. Expecting YYYY-mm-dd".to_string())])
                    }
                } else {
                    Ok(vec![Box::new("First date could not be parsed. Expecting YYYY-mm-dd".to_string())])
                }
            }
}

fn drop(params:&[&str],tx_request_from_main:&Sender<Request>,rx_output:&Receiver<Vec<Box<dyn Display + Send>>>)->Result<Vec<Box<dyn Display + Send>>,Box<dyn std::error::Error>>{
            if params.len() == 2 {
                let action = params[0];
                let date = params[1];
                if let Ok(date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                    match action {
                        "postpone" => {
                            tx_request_from_main.send(Request::Drop(DropType::Postpone, date))?;
                            Ok(rx_output.recv()?)
                        }
                        "collapse" => {
                            tx_request_from_main.send(Request::Drop(DropType::Collapse, date))?;
                            Ok(rx_output.recv()?)
                        }
                        "clean" => {
                            tx_request_from_main.send(Request::Drop(DropType::Clean, date))?;
                            Ok(rx_output.recv()?)
                        }
                        _ => Ok(
                            vec![Box::new("Second parameter must be \"postpone\", \"collapse\" or \"clean\". ".to_string())]
                        ),
                    }
                } else {
                    Ok(vec![Box::new("Date format must be YYYY-MM-DD".to_string())])
                }
            } else {
                Ok(vec![Box::new("Incorrect number of parameters".to_string())])
            }
}

fn get_help()->String{
    r#"Options:
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
help                                        - Display this text."#.to_string()
}