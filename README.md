# Language mutator
- Tool that makes transforming csv -> json and json -> csv easy

## Configuration
- `config.txt` in the root folder
- **Mode** - `json_to_csv` or `csv_to_json`
- **Source** - required fr both modes, can be either `*.json` or `*.csv` depending on the mode
- **Result 1** - required for both modes, can be either `*.json` or `*.csv` depending on the mode
- **Result 2** - required for `csv_to_json`, must be `*.json`

## Setup and running
1. Clone the repo
2. Install **Rust** [here](https://forge.rust-lang.org/infra/other-installation-methods.html)
3. Fill `config.txt` based on requirements specified in **Configuration** section
4. Run `cargo run` in the root directory of the repo
5. During **first run**, the program will download all necessary packages and run afterwards


## How to convert to Excel/CSV
1. First you need a JSON file with the mutations. The converter expects cs and en property to map mutations into different excel columns. The file can look something like this
```json
{
  "cs": 
    {
      "zdol": {
        "..."
      }
    },
    "en": {
        "zdol": {
            "..."
        }
    }
}
 ```
 2. in `config.txt` set mode to `json_to_csv`
 3. in `config.txt` set path to your JSON file
 4. in `config.txt` set path of your desired .csv output file
 5. run `cargo run`
 6. enjoy your .csv file

 config can look soomething like this

json_to_csv <br>

/path/to/your/source/file.json <br>
/path/to/your/output/file.csv <br>


## How to convert to CSV to JSON
 2. in `config.txt` set mode to `csv_to_json`
 3. in `config.txt` set path to your csv file
 4. in `config.txt` set path of your desired .json output file
 5. run `cargo run`
 6. Enjoy your .json file. If you need some quick converter to JS and your IDE does not help. This one seems to work the best for me https://www.convertonline.io/convert/json-to-js

  config can look soomething like this

csv_to_json <br>

/path/to/your/source/file.csv <br>
/path/to/your/output/file_cs.json <br>
/path/to/your/output/file_en.json <br>
