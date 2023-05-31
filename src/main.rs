use std::{process, io, env, path::Path};
use git2::{Repository, RemoteCallbacks, Cred, FetchOptions};
use dotenv::dotenv;
use std::io::{BufReader, BufRead, Write};
use std::fs::File;
use std::fs;

enum Action {
    MembershipsTsh,
    UsersOutput,
    UsersTsh
}

fn main() {
    dotenv().ok();

    loop {
        let mut option = String::new();
        println!("Choose an option: 1 - download repo, 2 - add user, 3 - set up new project repo, Q - quit");
        io::stdin().read_line(&mut option).unwrap();

        match option.to_string().trim() {
            "1" => download_repo(),
            "2" => add_user(),
            "3" => println!("setting up new repo"),
            // "4" => create_branch(),
            "5" => update_branch(),
            "q"|"Q" => process::exit(0x0100),
            opt => { println!("Invalid option: {}", opt); continue; }
        }
    }
}

fn add_user() {
    // update_branch();

    let mut first_name = String::new();
    let mut last_name = String::new();
    let mut email = String::new();

    println!("Provide user's first name:");
    io::stdin().read_line(&mut first_name).unwrap();
    println!("Provide user's last name:");
    io::stdin().read_line(&mut last_name).unwrap();
    println!("Provide user's email:");
    io::stdin().read_line(&mut email).unwrap();

    let first_name = first_name.trim().to_string();
    let last_name = last_name.trim().to_string();
    let email = email.trim().to_string();

    println!("Adding new user: {} {} - {}", first_name, last_name, email);
    create_branch(&first_name, &last_name);
    let membership_file_name = "tsh.tf".to_string();
    let membership_file_path = "gitlab/memberships".to_string();
    let membership_data = format!("
resource \"gitlab_group_membership\" \"tsh_{first_name}_{last_name}\" {{
  group_id     = var.gitlab_groups.tsh.id
  user_id      = var.gitlab_users.{first_name}_{last_name}.id
  access_level = \"developer\"
}}
");

    let users_output_file_name = "output.tf".to_string();
    let users_output_file_path = "gitlab/users".to_string();
    let users_output_data = format!("{}_{}", first_name.to_lowercase(), last_name.to_lowercase());

    let users_tsh_file_name = "tsh.tf".to_string();
    let users_tsh_file_path = "gitlab/users".to_string();
    let users_tsh_data = format!("
resource \"gitlab_user\" \"{}_{}\" {{
  name             = \"{} {}\"
  username         = \"{}.{}\"
  password         = \"\"
  email            = \"{}\"
  is_admin         = false
  can_create_group = false
  projects_limit   = 0
  reset_password   = true
}}", first_name.to_lowercase(), last_name.to_lowercase(), first_name, last_name, first_name.to_lowercase(), last_name.to_lowercase(), email);

    write_to_repo(membership_data, membership_file_name, membership_file_path, Action::MembershipsTsh).unwrap();
    write_to_repo(users_output_data, users_output_file_name, users_output_file_path, Action::UsersOutput).unwrap();
    write_to_repo(users_tsh_data, users_tsh_file_name, users_tsh_file_path, Action::UsersTsh).unwrap();
}

fn add_repo() {

}

fn download_repo() {
    let fetch_options = set_credentials();
    let mut builder = git2::build::RepoBuilder::new();

    builder.fetch_options(fetch_options);

    builder.clone(
        env::var("GITLAB_URL").unwrap().as_str(),
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
    ).unwrap();
    
    println!("Finished cloning");
}

fn create_branch(first_name: &String, last_name: &String) {
    let branch_name = format!("add-user-{}-{}", first_name.to_lowercase(), last_name.to_lowercase());

    let repo = Repository::open(
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
    ).unwrap();

    let object = repo.revparse_single("master").unwrap();
    let commit = object.as_commit().unwrap();

    repo.branch(&branch_name, &commit, true).unwrap();

    let obj = repo.revparse_single(&("refs/heads/".to_owned() + 
        &branch_name)).unwrap();

    repo.checkout_tree(
        &obj,
        None
    ).unwrap();

    repo.set_head(&("refs/heads/".to_owned() + &branch_name)).unwrap();

    println!("Set up new branch: {}", &branch_name);
}

fn update_branch() {
    let mut fetch_options = set_credentials();

    let repo = Repository::open(
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
    ).unwrap();

    repo.find_remote("origin").unwrap().fetch(&["master"], Some(&mut fetch_options), None).unwrap();

    println!("Master branch has been updated.");
}

fn set_credentials() -> FetchOptions<'static> {
    let mut callbacks = RemoteCallbacks::new();
      callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
          username_from_url.unwrap(),
          None,
          Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
          Some(env::var("PASSPHRASE").unwrap().as_str()),
        )
      });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    fetch_options
}

fn write_to_repo(passed_data: String, file_name: String, file_path: String, action: Action) -> Result<(), std::io::Error>{
    let base = format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap());
    let file_path = format!("{}/{}/{}", base, file_path, &file_name);
    println!("{}", file_path); 
    let existing_file = read_lines(&file_path)?;
    let file = File::create(format!("{}.tmp", file_path)).unwrap();
    let mut data = vec![];
    
    match action {
        Action::MembershipsTsh => { 
            for line in existing_file {
                data.push(line?);
            }
            data.push(passed_data);
        },
        Action::UsersOutput => {
            for line in existing_file {
                data.push(find_maciej(line?, &passed_data));
            }
        },
        Action::UsersTsh => {
            for line in existing_file {
                data.push(line?);
            }
            data.push(passed_data);
        }
    }

    process_file(data.join("\n"), file, file_path)?;
    println!("Successfully wrote to {}", file_name);
    Ok(())
}

fn find_maciej(line: String, passed_data: &String) -> String {
    match line {
        x if x.contains("maciejsajdok") => format!("    maciejsajdok           = {{ id = gitlab_user.maciejsajdok.id }}
    {passed_data}\t= {{ id = gitlab_user{passed_data}.id }}", ),
        _ => line
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<BufReader<File>>>
where P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn process_file(joined_data: String, mut file: File, file_path: String) -> Result<(), std::io::Error> {
        file.write_all(joined_data.as_bytes())?;
        fs::remove_file(format!("{}", &file_path))?;
        fs::rename(format!("{}.tmp", file_path), file_path)?;
        Ok(())
}
