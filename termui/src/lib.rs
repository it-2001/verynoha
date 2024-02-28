use std::io::{stdin, stdout, Read, Write};

#[derive(Clone, Debug, PartialEq)]
pub enum Confirm {
    Yes,
    No,
    Cancel,
}

impl Confirm {
    pub const OPTIONS: [Confirm; 3] = [Confirm::Yes, Confirm::No, Confirm::Cancel];

    pub fn as_str(&self) -> &'static str {
        match self {
            Confirm::Yes => "Yes",
            Confirm::No => "No",
            Confirm::Cancel => "Cancel",
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            Confirm::Yes => true,
            _ => false,
        }
    }

    pub fn default() -> Confirm {
        Confirm::Cancel
    }

    pub fn prompt(default: &Self) -> Self {
        let prompt = format!(
            "Is this correct? ({}/{}) [{}]",
            Confirm::Yes.as_str(),
            Confirm::No.as_str(),
            default.as_str()
        );
        let mut input = String::new();
        println!("{}", prompt);
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            return default.clone();
        }
        for option in Confirm::OPTIONS.iter() {
            if match_word(input, option.as_str(), None) {
                return option.clone();
            }
        }
        Confirm::Cancel
    }

    pub fn yesno(default: Option<bool>) -> bool {
        let default_str = match default {
            Some(true) => format!(" [{}]", Confirm::Yes.as_str()),
            Some(false) => format!(" [{}]", Confirm::No.as_str()),
            None => String::new(),
        };
        let mut prompt = format!(
            "Is this correct? ({}/{}){}",
            Confirm::Yes.as_str(),
            Confirm::No.as_str(),
            default_str
        );
        let mut input = String::new();
        loop {
            println!("{}", prompt);
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if input.is_empty() {
                match default {
                    Some(def) => return def,
                    None => {
                        prompt = format!(
                            "Please answer with ({}/{})",
                            Confirm::Yes.as_str(),
                            Confirm::No.as_str()
                        );
                        continue;
                    }
                }
            }
            if match_word(input, Confirm::Yes.as_str(), None) {
                return true;
            }
            if match_word(input, Confirm::No.as_str(), None) {
                return false;
            }
            prompt = format!(
                "Please answer with ({}/{})",
                Confirm::Yes.as_str(),
                Confirm::No.as_str()
            );
        }
    }
}

pub fn match_word(input: &str, word: &str, index: Option<usize>) -> bool {
    let trim = input.trim();
    match (trim.parse::<usize>(), index) {
        (Ok(num), Some(index)) => {
            if num == index {
                return true;
            }
        }
        _ => (),
    }
    trim.chars()
        .zip(word.trim().chars())
        .all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
}

pub fn input() -> String {
    let _ = stdout().flush();
    let mut result = String::new();
    while let Err(err) = stdin().read_line(&mut result) {
        println!("An unexpected error occured: {}", err);
        println!("Please try again:");
        result.clear();
    }
    result.trim().to_string()
}

pub fn try_input() -> Option<String> {
    let result = input();
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

pub fn long_input(help: bool) -> String {
    let _ = stdout().flush();
    if help {
        println!("Submit: ctrl + Z (Windows) or ctrl + D (linux)")
    }
    let mut result = String::new();
    while let Err(err) = stdin().read_to_string(&mut result) {
        println!("An unexpected error occured: {}", err);
        println!("Please try again:");
        result.clear();
    }
    result.trim().to_string()
}

pub fn input_uint() -> u64 {
    let _ = stdout().flush();
    loop {
        let num = input();
        match num.parse::<u64>() {
            Ok(num) => {
                return num;
            }
            Err(_) => {
                println!("Please enter a positive integer (0, 1, 2, 5, 9, 10, 99999999)")
            }
        }
    }
}

pub fn try_uint() -> Option<u64> {
    let _ = stdout().flush();
    let num = input();
    match num.parse::<u64>() {
        Ok(num) => Some(num),
        Err(_) => None,
    }
}

pub fn input_int() -> i64 {
    let _ = stdout().flush();
    loop {
        let num = input();
        match num.parse::<i64>() {
            Ok(num) => {
                return num;
            }
            Err(_) => {
                println!("Please enter an integer (-5, -1, 0, 1, 2, 3, 99999)")
            }
        }
    }
}

pub fn try_int() -> Option<i64> {
    let _ = stdout().flush();
    let num = input();
    match num.parse::<i64>() {
        Ok(num) => Some(num),
        Err(_) => None,
    }
}

pub fn input_number() -> f64 {
    let _ = stdout().flush();
    loop {
        let num = input();
        match num.parse::<f64>() {
            Ok(num) => {
                return num;
            }
            Err(_) => {
                println!("Please enter a number (-6.9, 0, 1.2, 10, 999999)")
            }
        }
    }
}

pub fn try_number() -> Option<f64> {
    let _ = stdout().flush();
    let num = input();
    match num.parse::<f64>() {
        Ok(num) => Some(num),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn please_say_yes() {
        let yeses = vec![
            "yes", "YES", "yEs", "YeS", "yES", "Yes", "y", "Y", "yEs", "yeS",
        ];
        for yes in yeses {
            assert_eq!(match_word(yes, Confirm::Yes.as_str(), None), true);
        }
    }

    #[test]
    fn please_say_no() {
        let nos = vec!["no", "NO", "nO", "No", "n", "N"];
        for no in nos {
            assert_eq!(match_word(no, Confirm::No.as_str(), None), true);
        }
    }
}
