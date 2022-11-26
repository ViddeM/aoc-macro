use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, LitInt, Token};

extern crate proc_macro;

#[proc_macro]
pub fn generate_day(day_year: TokenStream) -> TokenStream {
    let day_year =
        parse_macro_input!(day_year with Punctuated::<LitInt, Token![,]>::parse_terminated)
            .into_iter()
            .map(|l| l.base10_parse().expect("Failed to parse number"))
            .collect::<Vec<u32>>();
    if day_year.len() != 2 {
        panic!("Invalid number of arguments provided, expected 2 a day and a year");
    }
    let year = day_year.first().expect("No day provided");
    let day = day_year.get(1).expect("No year provided");

    TokenStream::from(quote! {
        use std::{
            env,
            fs::{self, File},
            io::Write,
            path::Path,
        };

        use reqwest::blocking::Client;
        use solution::{solve_part_one, solve_part_two};

        const YEAR: u32 = #year;
        const DAY: u32 = #day;
        const SESSION_COOKIE_FILE: &str = "/home/vidde/.aoc_session_cookie";

        enum Part {
            One,
            Two,
        }

        impl Part {
            fn from_env() -> Self {
                match env::var("part")
                    .expect("Failed to read 'part' environment variable")
                    .as_str()
                {
                    "part1" => Self::One,
                    "part2" => Self::Two,
                    other => panic!("Unexpected part {}", other),
                }
            }
        }

        const INPUT_FILE_PATH: &str = "./input.txt";

        fn download_or_read_input() -> String {
            // Try to read test file
            let test_file = match env::var("test_file") {
                Ok(file_path) => {
                    println!("Reading from test input file {}", file_path);
                    Some(fs::read_to_string(file_path).expect("Failed to read test data file"))
                }
                Err(_) => None,
            };

            if let Some(data) = test_file {
                return data;
            }

            // No test input file provided, read real data
            let file_path = Path::new(INPUT_FILE_PATH);

            if file_path.exists() {
                // File exists, return its content
                fs::read_to_string(file_path).expect("Failed to read input file")
            } else {
                // The file doesn't exist, download it.
                println!("File doesn't exist, downloading...");
                let data = download_input_data();
                let mut file = File::create(file_path).expect("Failed to create input file");
                file.write_all(data.as_bytes())
                    .expect("Failed to write downloaded input to file");
                data
            }
        }

        fn read_session_cookie() -> String {
            let session_cookie =
                fs::read_to_string(SESSION_COOKIE_FILE).expect("Failed to read session cookie file");

            session_cookie.trim_end().to_string()
        }

        fn download_input_data() -> String {
            let session_cookie = read_session_cookie();

            let url = format!("https://adventofcode.com/{}/day/{}/input", YEAR, DAY);

            Client::new()
                .get(&url)
                .header("cookie", format!("session={}", session_cookie))
                .send()
                .expect("Failed to retrieve advent of code input")
                .text()
                .expect("Failed to read text response from aoc webiste input")
        }

        fn handle_day() {
            let input = download_or_read_input();

            let solution = match Part::from_env() {
                Part::One => solve_part_one(&input),
                Part::Two => solve_part_two(&input),
            };
            println!("{}", solution);
        }

    })
}
