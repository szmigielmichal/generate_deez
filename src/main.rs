use std::{process, io, env, path::Path};
use git2::{Repository, RemoteCallbacks, Cred};
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    loop {
        let mut option = String::new();
        println!("Choose an option: 1 - download repo, 2 - add user, 3 - set up new project repo, Q - quit");
        io::stdin().read_line(&mut option).unwrap();

        match option.to_string().trim() {
            "1" => download_repo(),
            "2" => println!("adding user"),
            "3" => println!("setting up new repo"),
            "4" => branch(),
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
    let mut callbacks = RemoteCallbacks::new();
      callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
          username_from_url.unwrap(),
          None,
          Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
          Some(env::var("PASSPHRASE").unwrap().as_str()),
        )
      });

      // Prepare fetch options.
      let mut fo = git2::FetchOptions::new();
      fo.remote_callbacks(callbacks);

      // Prepare builder.
      let mut builder = git2::build::RepoBuilder::new();
      builder.fetch_options(fo);

      // Clone the project.
      builder.clone(
        env::var("GITLAB_URL").unwrap().as_str(),
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
      ).unwrap();
    
    println!("Finished cloning");
}

fn branch() {
    let branch_name = "new-branch";
    let repo = Repository::open(
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
    ).unwrap();

    let object = repo.revparse_single("master").unwrap();
    let commit = object.as_commit().unwrap();

    repo.branch(&branch_name, &commit, true).unwrap();

    let obj = repo.revparse_single(&("refs/heads/".to_owned() + 
        branch_name)).unwrap();

    repo.checkout_tree(
        &obj,
        None
    ).unwrap();

    repo.set_head(&("refs/heads/".to_owned() + branch_name)).unwrap();
}


