//source: https://doc.rust-lang.org/cargo/commands/cargo-install.html
// This command manages Cargo’s local set of installed binary crates. Only packages which have executable [[bin]] or [[example]] targets can be installed, and all executables are installed into the installation root’s bin folder.
//
// The installation root is determined, in order of precedence:
//
//     --root option
//     CARGO_INSTALL_ROOT environment variable
//     install.root Cargo config value
//     CARGO_HOME environment variable
//     $HOME/.cargo
//

use colored::Colorize;
use std::io::Write;
use std::{collections::HashMap, env, fs, io::Read};

use clap::{command, Arg, ArgAction};

fn determine_pkgs_install_dir() -> Option<String> {
    let dirs = vec![env::var("CARGO_INSTALL_ROOT"), env::var("CARGO_HOME")];
    for dir in dirs.into_iter().flatten() {
        if fs::read_dir(&dir).is_ok() {
            return Some(dir);
        }
    }
    let home_cargo = env::var("HOME").ok()? + "/.cargo";
    if fs::read_dir(&home_cargo).is_ok() {
        Some(home_cargo)
    } else {
        None
    }
}

fn list_pkgs(ir: &str) -> Option<Vec<String>> {
    //Install Root Bin Directory
    let ir_bin = ir.to_owned() + "/bin";
    let Ok(ir_bin) = fs::read_dir(ir_bin) else{
        return None;
    };
    let names: Vec<_> = ir_bin
        .filter_map(|x| x.ok()?.file_name().to_str().map(str::to_string))
        .collect();
    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

fn get_descs(ir: &str) -> Option<HashMap<String, (String, String)>> {
    let ir_source = ir.to_owned() + "/registry/src";
    // println!("{ir_source}");
    let Ok(ir_source) = fs::read_dir(&ir_source) else{
        return None;
    };
    let mut map = HashMap::new();
    let re = regex::Regex::new(r"-\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
    for source_dir in ir_source {
        let Ok(source_dir) = source_dir else {continue};
        // println!("{:?}", source_dir.path());
        let Ok(source_dir) = fs::read_dir(source_dir.path()) else {continue;};
        for dir in source_dir {
            let Ok(dir) = dir else {continue;};
            let Some(pkg_name) = dir.file_name().to_str().map(str::to_string) else {continue;};
            let Ok(mut cargo_toml) =
                fs::File::open(dir.path().to_str()?.to_string() + "/Cargo.toml")
            else {continue;};
            //description = "simple access to proc/status info on unix"
            let mut cargo_toml_content = String::new();
            let Ok(_) = cargo_toml.read_to_string(&mut cargo_toml_content) else {continue;};

            let Some(start) = cargo_toml_content.find("description = \"") else {continue;};
            let Some(end) = cargo_toml_content[start..].find('\n') else {continue;};
            let decs = &cargo_toml_content[start..start + end];
            let Some(start) = decs.find('\"') else { continue;};
            let Some(end) = decs.rfind('\"') else {continue;};
            let desc = &decs[start + 1..end];

            // println!("{pkg_name}: {desc}");
            let Some(split_c) = re.find(pkg_name.as_str()) else{continue;};
            let pkg_ver = &pkg_name[split_c.start() + 1..];
            let pkg_name = &pkg_name[..split_c.start()];

            map.insert(
                pkg_name.to_string(),
                (pkg_ver.to_string(), desc.to_string()),
            );

            // default-run = "btm";
            let find_start = "[[bin]]\nname = \"";
            // Find alt name for package
            let Some(start) = cargo_toml_content.find(find_start) else {continue;};
            let Some(end) = cargo_toml_content[start+find_start.len()..].find('\n') else {continue;};

            let alt_pkg_name =
                &cargo_toml_content[start + find_start.len()..start + find_start.len() + end - 1];

            map.insert(
                alt_pkg_name.to_string(),
                (pkg_ver.to_string(), desc.to_string()),
            );
            // println!("{pkg_name}(v: {pkg_ver}): {desc}");
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

fn get_desc(pkg: &String, map: &HashMap<String, (String, String)>) -> Option<(String, String)> {
    map.get(pkg).cloned()
}

fn main() {
    let ms = command!()
        .arg(
            Arg::new("versions")
                .short('v')
                .long("versions")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("descriptions")
                .short('d')
                .long("descriptions")
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    let print_versions = ms.get_flag("versions");
    let print_descs = ms.get_flag("descriptions");

    let Some(install_root) = determine_pkgs_install_dir() else{
          println!("Failed to locate cargo root.");
          std::process::exit(1);
    };
    // println!("[INFO] Install Root: {install_root}");
    let Some(pkgs) = list_pkgs(&install_root) else{
        panic!("Failed to list packages.");
    };
    let Some(map) = get_descs(&install_root) else{
        panic!("Failed to get descriptions.");
    };
    for pkg in pkgs {
        let (mut ver, mut desc) = match get_desc(&pkg, &map) {
            Some(o) => o,
            None => (String::from("n/a"), String::from("n/a")),
        };
        if print_versions {
            ver = format!(" {}", ver.yellow());
        } else {
            ver = String::new();
        }
        if print_descs {
            desc = format!(" {}", desc.blue());
        } else {
            desc = String::new();
        }
        print!("{}{}{} ", pkg.green().bold(), ver, desc);
        if print_descs || print_versions {
            println!();
        }
    }

    if !(print_descs || print_versions) {
        println!();
    }
}
