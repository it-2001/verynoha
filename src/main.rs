use termui::*;


#[derive(Clone, Debug)]
pub struct PlayerData {
    name: String,
    hash: u128,
}

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

    println!("Tell me a story!");
    let story = long_input(true);
    println!("I remember it all: {story}");

    print!("Gimme a number: ");
    match try_number() {
        Some(num) => println!("Thanks, you can have it back: {num}"),
        None => println!("Nevermind"),
    }
    
}
