use git2::{Repository, DiffDelta, DiffHunk, DiffLine, FetchOptions, RemoteCallbacks, Cred};
use std::{io, env, path::Path};
use std::io::{BufReader, BufRead, Write};
use std::fs::File;
use std::fs; 

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<BufReader<File>>>
where P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn process_file(joined_data: String, mut file: File, file_path: String) -> Result<(), std::io::Error> {
        file.write_all(joined_data.as_bytes())?;
        fs::remove_file(format!("{}", &file_path))?;
        fs::rename(format!("{}.tmp", file_path), file_path)?;
        Ok(())
}

pub fn print_diff() {
    let repo = Repository::open(
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
    ).unwrap();

    let index = repo.index().unwrap();

    let diff = repo.diff_index_to_workdir(Some(&index), None).unwrap();

    diff.print(git2::DiffFormat::Patch, |d, h, l| print_diff_line(d,h,l)).unwrap();
}

fn print_diff_line(
    _delta: DiffDelta,
    _hunk: Option<DiffHunk>,
    line: DiffLine,
) -> bool {
    match line.origin() {
        '+' | '-' | ' ' => print!("{}", line.origin()),
        _ => {}
    }
    print!("{}", std::str::from_utf8(line.content()).unwrap());
    true
}

pub fn set_credentials() -> FetchOptions<'static> {
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

pub fn open_repo() -> Repository {
    let repo = Repository::open(
        Path::new(&format!("{}/Code/infrastructure-as-code", env::var("HOME").unwrap()))
    ).unwrap();
    repo
}

