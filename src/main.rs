use std::fs::File;
use std::io::{Read, Write};
use csv::{Reader, Writer};
use serde_json::{json, Map, Result, Value};
use serde::Deserialize;
use regex::Regex;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;


#[derive(Debug, Deserialize)]
struct Mutation {
    key: String,
    value: String,
}

fn get_input(source: String) -> String{
    let mut file = File::open(source).expect("Failed to open file");

    // Read the JSON data into a String
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).expect("Failed to read file");
    json_str
}

fn parse_value(input: &Value) -> String {
    let value = str::replace(&input.as_str().unwrap().to_owned(), "\n", "");
    let value = str::replace(&value, "\r", "");
    let value = str::replace(&value, "﻿", "");

    let re = Regex::new(r"\s+").unwrap();
    let res = re.replace_all(&*value, " ").to_string();
    return res;

}

fn parse_string(input: String) -> String {
    let value = str::replace(&input, "\n", "");
    let value = str::replace(&value, "\r", "");
    let value = str::replace(&value, "﻿", "");

    let re = Regex::new(r"\s+").unwrap();
    let res = re.replace_all(&*value, " ").to_string();
    return res;

}

// Recursively iterating over json - &Value
fn get_object_keys(data: &Value, prepend: String) -> Vec<Mutation> {
    let mut current_scope_values: Vec<Mutation> = Vec::new();

    for (key, value) in data.as_object().unwrap() {
        let key = if prepend != "" {
            format!("{prepend}.{key}")
        } else {
            key.to_owned()
        };

        if value.is_string() {
            current_scope_values.push(Mutation { key, value: parse_value(value)})
        } else if value.is_object() {
            current_scope_values.extend(get_object_keys(value, key))
        } else if value.is_array(){
            let mut index = 0;

            for line in value.as_array().unwrap() {
                for (key_j, value) in line.as_object().unwrap() {
                    // index is used to correctly parse from csv to json
                    let array_key = format!("{key}.$.{index}.{key_j}");
                    current_scope_values.push(Mutation { key:  array_key, value: parse_value(value) });
                }
                index += 1;
            }
        }
    }

    current_scope_values
}

fn write_result(data: Vec<MutationToWrite>, path: String) -> Result<()> {
    let mut wtr = Writer::from_path(path).unwrap();
    wtr.write_record(&["key", "cs", "en"]).unwrap();

    for line in data.into_iter() {
        wtr.write_record(&[line.key, line.cs, line.en]).unwrap();
    }

    wtr.flush().unwrap();
    Ok(())
}

fn read_csv(path: String) -> (Vec<Mutation>, Vec<Mutation>) {
    let file = File::open(path).expect("Failed to open CSV file");

    let mut csv_reader = Reader::from_reader(file);
    let mut mutations_en: Vec<Mutation> = Vec::new();
    let mut mutations_cs: Vec<Mutation> = Vec::new();

    for result in csv_reader.deserialize::<MutationToWrite>() {
        match result {
            Ok(record) => {
                if !record.cs.is_empty(){
                    mutations_cs.push(Mutation { key: record.key.to_owned(), value: record.cs})
                }

                if !record.en.is_empty() {
                    mutations_en.push(Mutation { key: record.key, value: record.en})
                }
            }
            Err(err) => {
                eprintln!("Error reading CSV record: {}", err);
            }
        }
    }

    (mutations_cs, mutations_en)
}

// Used for merging objects in arrays
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
        let last_key = keys.last().expect("Invalid keys");

        match keys.iter().position(|&x| x == "$") {
            Some(index) => {
                let split = keys.split_at(index);
                path_keys = Vec::from(split.0);
                let array_keys = Vec::from(split.1);
                let last_array_key = array_keys.last().expect("Invalid keys");

                for key in path_keys.iter().take(path_keys.len() - 1) {
                    current_map = current_map
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Object(Map::new()))
                        .as_object_mut()
                        .expect("Object creation failed");
                }

                let json_key = path_keys.last().expect("").parse::<String>().unwrap();

                // Check if there already is an array
                match current_map.get(&*json_key) {
                    Some(value) => {
                        let mut current_items = value.as_array().unwrap().to_owned();
                        let index: usize =  array_keys[1].parse().unwrap();

                        if current_items.len() > index {
                            let mut found_object = current_items[index].to_owned();
                            merge(&mut found_object, json!({last_array_key.to_string(): parse_string(mutation.value)}));
                            let new_json = found_object;

                            current_items[index] = new_json;
                        } else {
                            // index is bigger than length but there is something already
                            if current_items.len() > 0 {
                                let delta = index - current_items.len();

                                for i in 0..(delta + 1) {
                                    current_items.push(json!({}));
                                }
                                current_items[index] = json!({last_array_key.to_string(): parse_string(mutation.value)});
                            } else {
                                for i in 0..(index + 1) {
                                    current_items.push(json!({}));
                                }
                                // println!("isnt object");
                                current_items[index] = json!({last_array_key.to_string(): parse_string(mutation.value)});
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

                        current_items[index] = json!({last_array_key.to_string(): parse_string(mutation.value)});
                        current_map.insert(json_key, Value::Array(current_items));
                    }
                }

            },
            None => {
                // We have simple string
                for key in keys.iter().take(keys.len() - 1) {
                    current_map = current_map
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Object(Map::new()))
                        .as_object_mut()
                        .expect("Failed object creation");
                }

                current_map.insert(last_key.to_string(), Value::String(parse_string(mutation.value)));
            },
        };

    }

    let mut file = File::create(path).expect("Failed to create file");

    // Serialize the JSON object to a JSON string
    let json_string = serde_json::to_string_pretty(&Value::Object(json_object))
        .expect("Formatting json failed");

    // Write the JSON string to the file
    file.write_all(json_string.as_bytes())
        .expect("Writing json failed");
}

fn get_config() -> (String, String, String, String) {
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
    let mut result_1 = String::new();
    let mut result_2 = String::new();

    let mut i = 0;

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if i == 0 {
                    mode = line;
                } else if i == 1 {
                    source = line;
                } else if i == 2 {
                    result_1 = line
                } else if i == 3 {
                    result_2 = line
                }
                i = i + 1;
            },
            Err(error) => panic!("Problem reading the file: {:?}", error),
        }
    }

    (mode, source, result_1, result_2)
}

#[derive(Debug, Deserialize)]
struct MutationMerge {
    cs: String,
    en: String
}


#[derive(Debug, Deserialize, Clone)]
struct MutationToWrite {
    key: String,
    cs: String,
    en: String,
}

fn main(){
    let (mode, source, result_1, result_2) = get_config();

    if mode == String::from("json_to_csv") {
        let source_json: Value = serde_json::from_str(&get_input(source)).unwrap();
        let v_cs: Value = source_json["cs"].clone();
        let v_en: Value = source_json["en"].clone();


        let res_cs =  get_object_keys(&v_cs, String::from(""));
        let res_en =  get_object_keys(&v_en, String::from(""));
        let mut mutations = HashMap::<String, MutationToWrite>::new();

        for line in res_cs {
            mutations.insert(line.key.clone(), MutationToWrite { key: line.key, cs: line.value, en: String::new()});
        }

        for line in res_en {
            match mutations.get(&line.key) {
                Some(found) => {
                    mutations.insert(line.key.clone(), MutationToWrite {key: line.key, cs: found.to_owned().cs.to_string(), en: line.value});
                },
                None => {
                    mutations.insert(line.key.clone(), MutationToWrite{key: line.key, cs: String::from(""), en: line.value});
                }
            }
        }

        let sorted = mutations.iter();
        let mut final_mutations: Vec<MutationToWrite> = Vec::new();

        for (key, value) in sorted {
            let owned = value.to_owned();
            final_mutations.push(owned)
        }

        let _ = write_result(final_mutations, result_1);
    } else if mode == String::from("csv_to_json"){
        let (mutations_cs, mutations_en) = read_csv(source);
        generate_json(mutations_cs, result_1);
        generate_json(mutations_en, result_2);
    }
}
