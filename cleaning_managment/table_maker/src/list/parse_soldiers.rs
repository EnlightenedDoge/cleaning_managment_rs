use serde_json::{self, Value,from_str};

#[derive(Debug)]
pub struct Soldier{
    pub name:String,
    pub phone:String,
}

pub fn parse(file:&str)->Result<Vec<Soldier>,Box<dyn std::error::Error>>{
    let data = std::fs::read_to_string(file)?;
    let json:Value=from_str(&data)?;
    let json_soldiers=json["soldiers"].as_array().expect("No \"soldiers\" array found in file");
    let mut soldiers = Vec::<Soldier>::new();
    for soldier in json_soldiers{
        soldiers.push(Soldier{
            name:soldier["Name"].as_str().expect("No soldier name found").to_string(),
            phone:soldier["Number"].as_str().expect("No soldier number found").to_string(),
        });
    }
    Ok(soldiers)
} 

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn parse_soldiers()
    {
        let s = r#"{
            "commanders": [
              {
                "Name": "***REMOVED*** ***REMOVED***"
              }
            ],
            "soldiers": [
              {
                "Name":"***REMOVED***",
                "Number":"***REMOVED***"
              }]
            }"#;
        std::fs::write("./test", s).expect("Failed to write to test file");
        let vec = vec![Soldier{name:"***REMOVED***".to_string(),phone:"***REMOVED***".to_string()}];
        let parsed = parse("./test").expect("failed test");
        assert_eq!(vec.len(),parsed.len());
        assert_eq!(vec[0].name,parsed[0].name);
        assert_eq!(vec[0].phone,parsed[0].phone);
        std::fs::remove_file("./test").expect("Could nout remove file");
    }
}