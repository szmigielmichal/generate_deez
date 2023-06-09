pub mod helpers;
use dotenv::dotenv;
use std::{process, io, env, path::Path, fs::File};
use crate::helpers::{read_lines, process_file, print_diff, set_credentials, open_repo, find_maciej};

enum Action {
    MembershipsTsh,
    UsersOutput,
    UsersTsh,

    ProjectsCicdVars,
    ProjectsProtectBranchesRule1,
    ProjectsShareWithGroupsTsh,
    ProjectsMockOutput,
    ProjectsOutput,
    ProjectsQiwa
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
            "3" => add_repo(),
            // "4" => create_branch(),
            "5" => update_master(),
            "6" => print_diff(),
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
    let branch_name = format!("add-user-{}-{}", first_name.to_lowercase(), last_name.to_lowercase());
    create_branch(branch_name);
    add_membership(&first_name, &last_name);
    add_users_output(&first_name, &last_name);
    add_users_tsh(&first_name, &last_name, &email);
}

fn add_membership(first_name: &String, last_name: &String) {
    let membership_file_name = "tsh.tf".to_string();
    let membership_file_path = "gitlab/memberships".to_string();
    let membership_data = format!("
resource \"gitlab_group_membership\" \"tsh_{first_name}_{last_name}\" {{
  group_id     = var.gitlab_groups.tsh.id
  user_id      = var.gitlab_users.{first_name}_{last_name}.id
  access_level = \"developer\"
}}
");
    write_to_repo(membership_data, membership_file_name, membership_file_path, Action::MembershipsTsh).unwrap();
}

fn add_users_output(first_name: &String, last_name: &String) {
    let users_output_file_name = "output.tf".to_string();
    let users_output_file_path = "gitlab/users".to_string();
    let users_output_data = format!("{}_{}", first_name.to_lowercase(), last_name.to_lowercase());
    write_to_repo(users_output_data, users_output_file_name, users_output_file_path, Action::UsersOutput).unwrap();
}

fn add_users_tsh(first_name: &String, last_name: &String, email: &String) {
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
    write_to_repo(users_tsh_data, users_tsh_file_name, users_tsh_file_path, Action::UsersTsh).unwrap();
}

fn add_repo() {
    let mut project_name = String::new();
    let mut description = String::new();

    println!("Provide project name (No need for hyphens or underscores):");
    io::stdin().read_line(&mut project_name).unwrap();
    println!("Provide project description:");
    io::stdin().read_line(&mut description).unwrap();

    let project_name = project_name.trim().to_string();
    let description = description.trim().to_string();
    
    println!("Adding new repo: {}, with description: {}", project_name, description);
    let branch_name = format!("{}", project_name.replace(" ", "-"));
    create_branch(branch_name);
    add_cicd_vars(&project_name);
    add_protect_branches(&project_name);
    add_share_groups(&project_name);
    add_mock_output(&project_name);
    add_projects_output(&project_name);
    add_projects_qiwa(&project_name, &description);
}

fn add_projects_qiwa(project_name: &str, description: &str) {
    let name_underscored = project_name.replace(" ", "_");
    let name_hyphened = project_name.replace(" ", "-");
    let data = format!("
resource \"gitlab_project\" \"{name_underscored}\" {{
  name                             = \"{name_hyphened}\"
  namespace_id                     = var.gitlab_groups.qiwa.id
  visibility_level                 = \"private\"
  request_access_enabled           = false
  shared_runners_enabled           = true
  default_branch                   = \"master\"
  description                      = \"{description}\"
  remove_source_branch_after_merge = false
  initialize_with_readme           = true
  public_builds                    = false

  lifecycle {{
    prevent_destroy = true
  }}
}} 
");
    let file_name = "cicd_vars_qiwa.tf".to_string();
    let file_path = "gitlab/projects/cicd_variables".to_string();

    write_to_repo(data, file_name, file_path, Action::ProjectsCicdVars).unwrap();
}

fn add_projects_output(project_name: &str) {
    todo!()
}

fn add_mock_output(project_name: &str) {
    todo!()
}

fn add_share_groups(project_name: &str) {
    todo!()
}

fn add_protect_branches(project_name: &str) {
    todo!()
}

fn add_cicd_vars(project_name: &str) {
    todo!()
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

fn create_branch(branch_name: String) {

    let repo = open_repo();

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

fn update_master() {
    let mut fetch_options = set_credentials();

    let repo = open_repo();

    repo.find_remote("origin").unwrap().fetch(&["master"], Some(&mut fetch_options), None).unwrap();

    println!("Master branch has been updated.");
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
        },
        Action::ProjectsCicdVars => todo!(),
        Action::ProjectsProtectBranchesRule1 => todo!(),
        Action::ProjectsShareWithGroupsTsh => todo!(),
        Action::ProjectsMockOutput => todo!(),
        Action::ProjectsOutput => todo!(),
        Action::ProjectsQiwa => todo!(),

    }

    process_file(data.join("\n"), file, file_path)?;
    println!("Successfully wrote to {}", file_name);
    Ok(())
}

