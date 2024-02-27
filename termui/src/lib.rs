

#[derive(Clone, Debug)]
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
            if match_word(input, option.as_str()) {
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
            if match_word(input, Confirm::Yes.as_str()) {
                return true;
            }
            if match_word(input, Confirm::No.as_str()) {
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

pub fn match_word(input: &str, word: &str) -> bool {
    input
        .trim()
        .chars()
        .zip(word.trim().chars())
        .all(|(a, b)|
            a.to_lowercase().eq(b.to_lowercase())
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn please_say_yes() {
        let yeses = vec!["yes", "YES", "yEs", "YeS", "yES", "Yes", "y", "Y", "yEs", "yeS"];
        for yes in yeses {
            assert_eq!(match_word(yes, Confirm::Yes.as_str()), true);
        }
    }

    #[test]
    fn please_say_no() {
        let nos = vec!["no", "NO", "nO", "No", "n", "N"];
        for no in nos {
            assert_eq!(match_word(no, Confirm::No.as_str()), true);
        }
    }
}
