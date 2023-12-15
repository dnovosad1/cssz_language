# Language mutator
- Tool that makes transforming csv -> json and json -> csv easy

## Configuration
- `config.txt` in the root folder
- **Mode** - `json_to_csv` or `csv_to_json`
- **Source** - required for both modes, can be either `*.json` or `*.csv` depending on the mode
- **Result 1** - required for both modes, can be either `*.json` or `*.csv` depending on the mode
- **Result 2** - required for `csv_to_json`, must be `*.json`

## Setup and running
1. Clone the repo
2. Install **Rust** [here](https://forge.rust-lang.org/infra/other-installation-methods.html)
3. Fill `config.txt` based on requirements specified in **Configuration** section
4. Run `cargo run` in the root directory of the repo
5. During **first run**, the program will download all necessary packages and run afterwards
