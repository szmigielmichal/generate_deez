use std::{process, io};

fn main() {
    loop {
        let mut option = String::new();
        println!("Choose an option: 1 - download repo, 2 - add user, 3 - set up new project repo, Q - quit");
        io::stdin().read_line(&mut option).unwrap();

        match option.to_string().trim() {
            "1" => println!("downloading repo"),
            "2" => println!("adding user"),
            "3" => println!("setting up new repo"),
            "q"|"Q" => process::exit(0x0100),
            opt => { println!("Invalid option: {}", opt); continue; }
        }
    }
}

fn add_user() {

}

fn add_repo() {

}

fn download_repo() {

}


