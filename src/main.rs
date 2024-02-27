use termui::*;

fn main() {
    println!("Tak teď se uvidí");

    let default = Confirm::default();
    let result = Confirm::prompt(&default);
    println!("You chose: {}", result.as_str());
    println!("Is it ok? {}", result.is_ok());

    let result = Confirm::yesno(Some(true));
    println!("You chose: {}", result.to_string());

    let result = Confirm::yesno(None);
    println!("You chose: {}", result.to_string());

}
