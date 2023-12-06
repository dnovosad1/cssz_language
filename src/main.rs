use std::fs::File;
use std::io::{Read, Write};
use csv::{Reader, Writer};
use serde_json::{json, Map, Result, Value};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Mutation {
    key: String,
    cs: String
}

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
    wtr.write_record(&["key", "cs"]).unwrap();

    for line in data.into_iter() {
        wtr.write_record(&[line.0, line.1]).unwrap();
    }

    wtr.flush().unwrap();
    Ok(())
}

fn read_csv() -> Vec<Mutation> {
    let file = File::open("/home/gandalfthegray/language/src/result.csv").expect("Failed to open CSV file");

    let mut csv_reader = Reader::from_reader(file);
    let mut mutations: Vec<Mutation> = Vec::new();

    for result in csv_reader.deserialize::<Mutation>() {
        match result {
            Ok(record) => {
                mutations.push(record);
            }
            Err(err) => {
                eprintln!("Error reading CSV record: {}", err);
            }
        }
    }

    mutations
}

fn generate_json(data: Vec<Mutation>) {
    let mut json_object = Map::new();

    for mutation in data {
        let mut current_map = &mut json_object;

        let keys: Vec<&str> = mutation.key.split('.').collect();
        let last_key = keys.last().expect("prazdny path kur*a");

        for key in keys.iter().take(keys.len() - 1) {
            current_map = current_map
                .entry(key.to_string())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .expect("neco se do*ebalo a nemuze to najit object");
        }

        current_map.insert(last_key.to_string(), Value::String(mutation.cs));
    }

    let mut file = File::create("/home/gandalfthegray/language/src/output.json").expect("Failed to create file");

    // Serialize the JSON object to a JSON string
    let json_string = serde_json::to_string_pretty(&Value::Object(json_object))
        .expect("serializace neprobehla");

    // Write the JSON string to the file
    file.write_all(json_string.as_bytes())
        .expect("jak expectuju tak expectuju nepada to");
}

fn main(){
    generate_json(read_csv());
    // let v: Value = serde_json::from_str(&get_input()).unwrap();
    //
    // let res =  get_object_keys(&v, String::from(""));
    // let _ = write_result(res);
}
