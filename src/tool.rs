use core::str;
use std::fmt::Debug;

pub fn upper_case(bytes: &[u8]) -> Vec<u8> {
    bytes
        .into_iter()
        .map(|byte| if *byte >= 97 { *byte - 32 } else { *byte })
        .collect::<Vec<u8>>()
}

pub fn upper_case_byte(byte: u8) -> u8 { 
    if byte >= 97 { byte - 32 } else { byte }
}

pub fn print_error<T: Debug>(data: &'_ [u8], error: &T, line: usize, column: usize, end: usize) {
    let mut line_index = 0;
    let mut start_index = 0;
    let mut end_index = data.len()-1;
    let mut line_found = false;

    for (index, byte) in data.iter().enumerate() {
        if *byte == b'\n' {
            line_index += 1;

            if line_index == line {
                start_index = index;
                line_found = true;
            }

            if line_found {
                end_index = index;
                break;
            }
        }
    }

    println!("");
    println!("Error: {:?}", &error);
    println!("Line: {}, column: {}", line + 1, column);
    println!("{}", str::from_utf8(&data[start_index..end_index]).unwrap());
    println!("{}{}", (0..column).map(|_| " ").collect::<String>(), (0..end-column).map(|_| "^").collect::<String>());
    println!("");
}
