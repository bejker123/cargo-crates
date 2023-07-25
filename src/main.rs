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
//source: https://doc.rust-lang.org/cargo/commands/cargo-install.html

use colored::Colorize;
use std::{collections::HashMap, env, fs, io::Read, process::exit};

fn determine_pkgs_install_dir() -> Vec<String> {
    //According to cargo documentation it's best to start looking for the Install Root Directory in
    //this order:
    let mut dirs = vec![
        env::var("CARGO_INSTALL_ROOT"),
        env::var("CARGO_HOME"),
        env::var("HOME").map(|x| x + "/.cargo"),
    ];
    dirs.dedup();
    dirs.iter()
        .flatten()
        .filter_map(|x| {
            if fs::read_dir(x).is_ok() {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect()
}

fn list_pkgs(ir: &str) -> Option<Vec<String>> {
    //Path to the Install Root Bin Directory
    let ir_bin = ir.to_owned() + "/bin";

    //If the dir doesn't exist return None.
    let Ok(ir_bin) = fs::read_dir(ir_bin) else{
        return None;
    };

    //Get binary names.
    let names: Vec<_> = ir_bin
        .filter_map(|x| x.ok()?.file_name().to_str().map(str::to_string))
        .collect();

    //This way it's easier to tell if the function failed.
    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

//Get information (version and description) about installed cargo packages.
fn get_pkgs_info(ir: &str) -> Option<HashMap<String, (String, String)>> {
    //Install Root Source Directory.
    let ir_source = ir.to_owned() + "/registry/src";

    //Check if it exists.
    let Ok(ir_source) = fs::read_dir(&ir_source) else{
        return None;
    };

    //Allocate an empty hashmap.
    let mut map = HashMap::new();

    //Create regex expression used to separate the version and the pkg name.
    //It's important to create the expression before the loop. Moving the creation here improved
    //the performence 3x.
    let re = regex::Regex::new(r"-\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();

    //For each valid directory in the Install Root Source Directory find it's child directories and look
    //for Cargo.toml files containing relevant package information.
    for source_dir in ir_source.into_iter().flatten() {
        //Check if dir exists.
        let Ok(source_dir) = fs::read_dir(source_dir.path()) else {continue;};

        for dir in source_dir.into_iter().flatten() {
            //Get the package name from path.
            let Some(pkg_name) = dir.file_name().to_str().map(str::to_string) else {continue;};

            //Check if Cargo.toml exists
            let Ok(mut cargo_toml) =
                fs::File::open(dir.path().to_str()?.to_string() + "/Cargo.toml")
            else {continue;};

            //Read the Cargo.toml file
            let mut cargo_toml_content = String::new();
            let Ok(_) = cargo_toml.read_to_string(&mut cargo_toml_content) else {continue;};

            //Get the package description
            let Some(start) = cargo_toml_content.find("description = \"") else {continue;};
            let Some(end) = cargo_toml_content[start..].find('\n') else {continue;};
            let decs = &cargo_toml_content[start..start + end];
            let Some(start) = decs.find('\"') else { continue;};
            let Some(end) = decs.rfind('\"') else {continue;};
            let desc = &decs[start + 1..end];

            //separate the package version and name.
            let Some(split_c) = re.find(pkg_name.as_str()) else{continue;};
            let pkg_ver = &pkg_name[split_c.start() + 1..];
            let pkg_name = &pkg_name[..split_c.start()];

            //Insert them into the hashmap.
            map.insert(
                pkg_name.to_string(),
                (pkg_ver.to_string(), desc.to_string()),
            );

            //Find alternative names for the package.
            let find_start = "[[bin]]\nname = \"";
            // Find alt name for package
            let Some(start) = cargo_toml_content.find(find_start) else {continue;};
            let Some(end) = cargo_toml_content[start+find_start.len()..].find('\n') else {continue;};

            let alt_pkg_name =
                &cargo_toml_content[start + find_start.len()..start + find_start.len() + end - 1];

            //Insert them into the hashmap.
            map.insert(
                alt_pkg_name.to_string(),
                (pkg_ver.to_string(), desc.to_string()),
            );
        }
    }
    //If the hashmap is empty return None.
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

#[derive(PartialEq, Debug)]
struct CliOptions {
    print_versions: bool,
    print_descs: bool,
    print_paths: bool,
}

impl CliOptions {
    //Unpack the struct into a tuple which is easier with a dedicated function.
    //Order:
    // print_versions
    // print_descs
    // print_paths
    pub fn unpack(&self) -> (bool, bool, bool) {
        (self.print_versions, self.print_descs, self.print_paths)
    }
}

fn print_help() -> ! {
    println!("Usage:");
    let options = "OPTIONS".yellow().bold();
    let call = format!("{} {}", "cargo".red(), "ls-crates".blue().bold());
    println!("{call} [{options}]");
    println!("{options}:");
    println!("\t-h --help print help");
    println!("\t-v print versions");
    println!("\t-d print descriptions");
    println!("{}:", "Examples".purple());
    println!("{call} -v - print package names and versions");
    println!("{call} -d - print package names and descriptions");
    println!("{call} -vd - print package names, descriptions and versions");
    println!("{call} -dv - print package names, descriptions and versions");
    println!("Note:\nInvalid arguments will be ignored.");
    std::process::exit(0)
}

//Accept an argument rather than get the cli args from the api to enable testing.
fn parse_args<T: ToString>(args: &[T]) -> CliOptions {
    let mut op = CliOptions {
        print_descs: false,
        print_versions: false,
        print_paths: false,
    };
    for arg in args.into_iter().skip(1) {
        let arg = arg.to_string();
        if arg == "-h" || arg == "--help" {
            print_help();
        }
        op.print_descs = arg.contains('d');
        op.print_versions = arg.contains('v');
        op.print_paths = arg.contains('p');
    }
    op
}

fn main() {
    let args: Vec<String> = env::args().collect();
    //Parse command line arguments
    let options = parse_args(&args);
    let (print_versions, print_descs, print_paths) = options.unpack();

    //Locate packages
    let install_dirs = determine_pkgs_install_dir();
    if install_dirs.is_empty() {
        panic!("Failed to locate cargo root.");
    };
    if print_paths {
        for dir in install_dirs {
            println!("{dir}");
        }
        exit(0);
    }

    let mut pkgs: Vec<String> = Vec::new();
    let mut map: HashMap<String, (String, String)> = HashMap::new();
    for dir in install_dirs {
        //Get the list of installed packages
        if let Some(mut pkgs_) = list_pkgs(&dir) {
            pkgs.append(&mut pkgs_);
        };
        //Get packages' descriptions and versions
        if let Some(map_) = get_pkgs_info(&dir) {
            map.extend(map_);
        };
    }

    if pkgs.is_empty() {
        panic!("Failed to list packages.");
    }
    if map.is_empty() {
        panic!("Failed to get info.");
    }

    //Print info out
    for mut pkg in pkgs {
        //Get package description
        let (mut ver, mut desc) = map
            .get(&pkg)
            .cloned()
            .map_or_else(|| (String::from("n/a"), String::from("n/a")), |o| o);
        if let Some(i) = pkg.find(".exe") {
            pkg = pkg[..i].to_owned()
        }

        //If user passed -v print version info, additionally if -d is passed print package
        //descriptions.
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

mod test {
    #[test]
    fn determine_pkgs_install_dir() {
        std::env::set_var("CARGO_INSTALL_ROOT", "tmp/CIR_PATH");
        std::env::set_var("CARGO_HOME", "tmp/CH_PATH");
        std::env::set_var("HOME", "tmp/H_PATH");
        std::fs::create_dir("tmp/").unwrap();
        std::fs::create_dir("tmp/CIR_PATH").unwrap();
        std::fs::create_dir("tmp/CH_PATH").unwrap();
        std::fs::create_dir_all("tmp/H_PATH/.cargo").unwrap();
        let out = crate::determine_pkgs_install_dir();
        assert_eq!(
            out,
            vec![
                String::from("tmp/CIR_PATH"),
                String::from("tmp/CH_PATH"),
                String::from("tmp/H_PATH/.cargo")
            ]
        );
        std::fs::remove_dir_all("tmp/").unwrap();
    }

    #[test]
    fn parse_args() {
        assert_eq!(
            crate::parse_args(&["vdp"]),
            crate::CliOptions {
                print_versions: true,
                print_descs: true,
                print_paths: true
            }
        );
        assert_eq!(
            crate::parse_args(&["vd"]),
            crate::CliOptions {
                print_versions: true,
                print_descs: true,
                print_paths: false
            }
        );
        assert_eq!(
            crate::parse_args(&["dp"]),
            crate::CliOptions {
                print_versions: false,
                print_descs: true,
                print_paths: true
            }
        );
        assert_eq!(
            crate::parse_args(&["vp"]),
            crate::CliOptions {
                print_versions: true,
                print_descs: false,
                print_paths: true
            }
        );
        assert_eq!(
            crate::parse_args(&[""]),
            crate::CliOptions {
                print_versions: false,
                print_descs: false,
                print_paths: false
            }
        );
        assert_eq!(
            crate::parse_args(&["asfv"]),
            crate::CliOptions {
                print_versions: true,
                print_descs: false,
                print_paths: false
            }
        );
    }
}
