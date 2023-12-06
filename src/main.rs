use std::fs::File;
use std::io::Read;
use csv::Writer;
use serde_json::{Result, Value};

fn get_input() -> String{
    let mut file = File::open("/home/gandalfthegray/language/src/input.json").expect("Failed to open file");

    // Read the JSON data into a String
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).expect("Failed to read file");
    json_str
}

fn get_object_keys(data: &Value, prepend: String) -> Vec<(String, String)> {
    let mut current_scope_values: Vec<(String, String)> = Vec::new();

    for (key, value) in data.as_object().unwrap() {
        let key = if prepend != "" {
            format!("{prepend}.{key}")
        } else {
            key.to_owned()
        };

        if value.is_string() {
            current_scope_values.push((key, value.as_str().unwrap().to_owned()))
        } else if value.is_object() {
            current_scope_values.extend(get_object_keys(value, key))
        }
    }

    current_scope_values
}

fn write_result(data: Vec<(String, String)>) -> Result<()> {
    let mut wtr = Writer::from_path("/home/gandalfthegray/language/src/result.csv").unwrap();

    for line in data.into_iter() {
        wtr.write_record(&[line.0, line.1]).unwrap();
    }

    wtr.flush().unwrap();
    Ok(())
}

fn main(){
    let v: Value = serde_json::from_str(&get_input()).unwrap();

    let res =  get_object_keys(&v, String::from(""));
    let _ = write_result(res);
}
