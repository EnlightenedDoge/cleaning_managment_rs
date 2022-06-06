use reqwest;
use serde::Serialize;

pub fn send_to(number: &str, message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let body = Body {
        key: "J61G***REMOVED***bp",
        user: "***REMOVED***",
        pass: "***REMOVED***",
        sender: "***REMOVED***",
        recipient: number,
        msg: message,
    };

    let res = //reqwest::blocking::Client::new()
    client
        .post("https://api.sms4free.co.il/ApiSMS/SendSMS")
        .json(&body)
        .send()?
        .status()
        .to_string();
    Ok(res)
}

#[derive(Serialize)]
struct Body<'a> {
    key: &'a str,
    user: &'a str,
    pass: &'a str,
    sender: &'a str,
    recipient: &'a str,
    msg: &'a str,
}
