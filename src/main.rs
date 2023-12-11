use std::fs::File;
use std::io::{Read, Write};
use csv::{Reader, Writer};
use serde_json::{json, Map, Result, Value};
use serde::Deserialize;
use regex::Regex;
use std::io::prelude::*;
use std::io::BufReader;


#[derive(Debug, Deserialize)]
struct Mutation {
    key: String,
    cs: String
}

fn get_input(source: String) -> String{
    let mut file = File::open(source).expect("Failed to open file");

    // Read the JSON data into a String
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).expect("Failed to read file");
    json_str
}

fn parse_string(input: &Value) -> String {
    let value = str::replace(&input.as_str().unwrap().to_owned(), "\n", "");
    let value = str::replace(&value, "\r", "");
    let value = str::replace(&value, "﻿", "");
    // let value = str::replace(&value, "\ufeff", "");

    let re = Regex::new(r"\s+").unwrap();
    let res = re.replace_all(&*value, " ").to_string();
    return res;

}

fn get_object_keys(data: &Value, prepend: String) -> Vec<Mutation> {
    let mut current_scope_values: Vec<Mutation> = Vec::new();

    for (key, value) in data.as_object().unwrap() {
        let key = if prepend != "" {
            format!("{prepend}.{key}")
        } else {
            key.to_owned()
        };

        if value.is_string() {
            current_scope_values.push(Mutation { key, cs: parse_string(value)})
        } else if value.is_object() {
            current_scope_values.extend(get_object_keys(value, key))
        } else if value.is_array(){
            let mut index = 0;
            for line in value.as_array().unwrap() {
                for (key_j, value) in line.as_object().unwrap() {
                    let array_key = format!("{key}.$.{index}.{key_j}");
                    // let value = str::replace(&*value.as_str().unwrap().to_owned(), "\n", "");
                    current_scope_values.push(Mutation { key:  array_key, cs: parse_string(value) });
                }
                index += 1;
            }
        }
    }

    current_scope_values
}

fn write_result(data: Vec<Mutation>, path: String) -> Result<()> {
    let mut wtr = Writer::from_path(path).unwrap();
    wtr.write_record(&["key", "cs", "en"]).unwrap();

    for line in data.into_iter() {
        wtr.write_record(&[line.key, line.cs, String::from("")]).unwrap();
    }

    wtr.flush().unwrap();
    Ok(())
}

fn read_csv(path: String) -> Vec<Mutation> {
    let file = File::open(path).expect("Failed to open CSV file");

    let mut csv_reader = Reader::from_reader(file);
    let mut mutations: Vec<Mutation> = Vec::new();

    for result in csv_reader.deserialize::<Mutation>() {
        match result {
            Ok(record) => {
                if !record.cs.is_empty() {
                    mutations.push(record);
                }
            }
            Err(err) => {
                eprintln!("Error reading CSV record: {}", err);
            }
        }
    }

    mutations
}

fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                }
                else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }

            return;
        }
    }

    *a = b;
}

fn generate_json(data: Vec<Mutation>, path: String) {
    let mut json_object = Map::new();

    for mutation in data {
        let mut current_map = &mut json_object;

        let keys: Vec<&str> = mutation.key.split('.').collect();
        let mut path_keys: Vec<&str> = Vec::new();
        let last_key = keys.last().expect("prazdny path kur*a");
        let last_array_key =

        match keys.iter().position(|&x| x == "$") {
            Some(index) => {
                let split = keys.split_at(index);
                path_keys = Vec::from(split.0);
                // println!("Path keys {:?}", path_keys);
                let array_keys = Vec::from(split.1);
                let last_array_key = array_keys.last().expect("Kurva co to je");

                for key in path_keys.iter().take(path_keys.len() - 1) {
                    current_map = current_map
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Object(Map::new()))
                        .as_object_mut()
                        .expect("neco se do*ebalo a nemuze to najit object");
                }

                let json_key = path_keys.last().expect("").parse::<String>().unwrap();

                // Check if there already is an array
                match current_map.get(&*json_key) {
                    Some(value) => {
                        let mut current_items = value.as_array().unwrap().to_owned();
                        let index: usize =  array_keys[1].parse().unwrap();

                        if current_items.len() > index {
                            let mut found_object = current_items[index].to_owned();
                            merge(&mut found_object, json!({last_array_key.to_string(): parse_string(&Value::String(mutation.cs))}));
                            let new_json = found_object;

                            current_items[index] = new_json;
                        } else {
                            // index is bigger than length but there is something already
                            if current_items.len() > 0 {
                                let delta = index - current_items.len();

                                for i in 0..(delta + 1) {
                                    current_items.push(json!({}));
                                }
                                current_items[index] = json!({last_array_key.to_string(): parse_string(&Value::String(mutation.cs))});
                            } else {
                                for i in 0..(index + 1) {
                                    current_items.push(json!({}));
                                }
                                // println!("isnt object");
                                current_items[index] = json!({last_array_key.to_string(): parse_string(&Value::String(mutation.cs))});
                            }
                        }

                        current_map.insert(json_key, Value::Array(current_items));
                    },
                    None => {
                        let mut current_items = Vec::new();
                        let index: usize =  array_keys[1].parse().unwrap();
                        for i in 0..(index + 1) {
                            current_items.push(json!({}));
                        }

                        current_items[index] = json!({last_array_key.to_string(): parse_string(&Value::String(mutation.cs))});
                        current_map.insert(json_key, Value::Array(current_items));
                    }
                }

            },
            None => {
                for key in keys.iter().take(keys.len() - 1) {
                    current_map = current_map
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Object(Map::new()))
                        .as_object_mut()
                        .expect("neco se do*ebalo a nemuze to najit object");
                }

                current_map.insert(last_key.to_string(), Value::String(parse_string(&Value::String(mutation.cs))));
            },
        };
        // println!("We are parsing an array");

    }

    let mut file = File::create(path).expect("Failed to create file");

    // Serialize the JSON object to a JSON string
    let json_string = serde_json::to_string_pretty(&Value::Object(json_object))
        .expect("serializace neprobehla");

    // Write the JSON string to the file
    file.write_all(json_string.as_bytes())
        .expect("jak expectuju tak expectuju nepada to");
}

fn get_config() -> (String, String, String) {
    let file = File::open("config.txt");

    let file = match file {
        Ok(file) => file,
        Err(error) => {
            panic!("Problem opening the file: {:?}", error)
        },
    };

    let reader = BufReader::new(file);

    let mut mode = String::new();
    let mut source = String::new();
    let mut result = String::new();

    let mut i = 0;

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if i == 0 {
                    mode = line;
                } else if i == 1 {
                    source = line;
                } else if i == 2 {
                    result = line
                }
                i = i + 1;
            },
            Err(error) => panic!("Problem reading the file: {:?}", error),
        }
    }

    (mode, source, result)
}

fn main(){
    let (mode, source, result) = get_config();

    if mode == String::from("json_to_csv") {
        let v: Value = serde_json::from_str(&get_input(source)).unwrap();

        let res =  get_object_keys(&v, String::from(""));
        let _ = write_result(res, result);
    } else if mode == String::from("csv_to_json"){
        generate_json(read_csv(source), result);
    }
}
