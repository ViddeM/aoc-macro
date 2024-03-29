#![feature(test)]

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

        #[cfg(feature = "download_input")]
        use reqwest::blocking::Client;

        use solution::{solve_part_one, solve_part_two, parse};

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
                // The file doesn't exist and we have the download_input feature enabled, download it.
                let data = download_input_data();
                let mut file = File::create(file_path).expect("Failed to create input file");
                file.write_all(data.as_bytes())
                    .expect("Failed to write downloaded input to file");
                data
            }
        }

        fn read_session_cookie() -> String {
            let session_cookie_path = match env::var("SESSION_COOKIE_FILEPATH") {
                Ok(fp) => fp,
                Err(_) => SESSION_COOKIE_FILE.to_string(),
            };
            
            let session_cookie =
                fs::read_to_string(session_cookie_path).expect("Failed to read session cookie file");

            session_cookie.trim_end().to_string()
        }

        #[cfg(not(feature = "download_input"))]
        fn download_input_data() -> String {
            panic!("No input provided, maybe enable the `download_input` feature?");
        }

        #[cfg(feature = "download_input")]
        fn download_input_data() -> String {
            println!("File doesn't exist, downloading...");
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
                Part::One => solve_part_one(parse(&input)),
                Part::Two => solve_part_two(parse(&input)),
            };
            println!("{}", solution);
        }

        #[cfg(test)]
        mod test {
            extern crate test;
            use test::Bencher;
            use crate::download_or_read_input;
            use crate::solution::{solve_part_one, solve_part_two, parse};

            #[bench]
            fn bench_parse(b: &mut Bencher) {
                let input = download_or_read_input();
                b.iter(|| parse(&input))
            }

            #[bench]
            fn bench_part_1(b: &mut Bencher) {
                let input = download_or_read_input();
                b.iter(|| solve_part_one(parse(&input)))
            }

            #[bench]
            fn bench_part_2(b: &mut Bencher) {
                let input = download_or_read_input();
                b.iter(|| solve_part_two(parse(&input)))
            }
        }
    })
}
