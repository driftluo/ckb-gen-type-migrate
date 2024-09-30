use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self};
use std::io::{self, BufRead, BufReader, Cursor, Write};
use std::process::Command;

#[derive(Deserialize, Serialize, Clone, Debug)]
struct CargoInfo {
    reason: String,
    package_id: String,
    manifest_path: String,
    target: Target,
    message: Message,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
struct Target {
    kind: Vec<String>,
    crate_types: Vec<String>,
    name: String,
    src_path: String,
    edition: String,
    doc: bool,
    doctest: bool,
    test: bool,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
struct Message {
    rendered: String,
    // r#$message_type: String,
    children: Vec<Children>,
    code: Code,
    level: String,
    message: String,
    spans: Vec<Span>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
struct Children {
    children: Vec<Children>,
    // code:
    level: String,
    message: String,
    // rendered:
    spans: Vec<Span>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
struct Code {
    code: String,
    explanation: String,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
struct Span {
    byte_end: usize,
    byte_start: usize,
    column_end: usize,
    column_start: usize,
    // expansion: Option<String>,
    file_name: String,
    is_primary: bool,
    // label: Option<String>,
    line_end: usize,
    line_start: usize,
    // suggested_replacement: Option<String>,
    // suggestion_applicability: Option<String>,
    text: Vec<Text>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Text {
    highlight_end: usize,
    highlight_start: usize,
    text: String,
}

fn main() {
    let matches = clap::Command::new("ckb-gen-type-migrate")
        .name("CKB Gen Type Migrate ")
        .about("Help migrate breaking changes in molecule code")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::new("cargo")
                .long("cargo")
                .help("Whether to run cargo by default, default is true")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("number")
                .long("number")
                .short('n')
                .default_value("10")
                .help("Number of times cargo was run, default is 10")
                .value_parser(clap::value_parser!(usize))
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    let is_cargo = matches.get_flag("cargo");
    let number: usize = *matches.get_one("number").unwrap();

    for _ in 0..number {
        run(is_cargo);
    }
}

fn run(is_cargo: bool) {
    let inputs: Box<dyn Iterator<Item = String>> = if is_cargo {
        let output = Command::new("cargo")
            .args(["check", "--tests", "--message-format", "json"])
            .output()
            .expect("can't execute cargo check now");
        Box::new(
            BufReader::new(Cursor::new(output.stdout))
                .lines()
                .map(|l| l.unwrap()),
        )
    } else {
        Box::new(io::stdin().lines().map(|l| l.unwrap()))
    };
    let mut res = Vec::new();

    for line in inputs {
        match serde_json::from_str::<CargoInfo>(&line) {
            Ok(a) => {
                if a.reason == "compiler-message" {
                    res.push(a)
                }
            }
            Err(_) => {}
        }
    }

    let re = regex::Regex::new(r"\.(pack|into|unpack)\(\)").unwrap();
    let re_default = regex::Regex::new(r"Default").unwrap();
    let re_default_type = regex::Regex::new(r"<(.*)>").unwrap();
    let mut x: HashMap<String, HashMap<usize, CargoInfo>> = HashMap::new();
    let mut y: HashMap<String, HashSet<usize>> = HashMap::new();
    for each in res {
        let span = each.message.spans.last().unwrap();
        // println!("{}, {}, {:?}", span.file_name, span.line_start, span.text);

        let code = &span.text[0].text;
        if re.is_match(code) {
            if y.entry(span.file_name.clone())
                .or_default()
                .insert(span.line_start)
            {
                x.entry(span.file_name.clone())
                    .or_default()
                    .insert(span.line_start, each.clone());
            }
        } else if re_default.is_match(code) {
            if y.entry(span.file_name.clone())
                .or_default()
                .insert(span.line_start)
            {
                x.entry(span.file_name.clone())
                    .or_default()
                    .insert(span.line_start, each.clone());
            }
            // let m = re_default_type
            //     .find(&each.message.children[0].message)
            //     .unwrap();
            // println!(
            //     "{}, {}, {}",
            //     code,
            //     &m.as_str()[1..m.len() - 1],
            //     re_default.replace(&code, &m.as_str()[1..m.len() - 1])
            // );
        }
    }
    // return;
    use fs::OpenOptions;
    for _ in 0..3 {
        for (path, info) in x.iter() {
            let mut new_content = Vec::new();
            let file = OpenOptions::new().read(true).open(&path).unwrap();

            let old_buf = BufReader::new(file);

            for (index, line) in old_buf.lines().enumerate().map(|(i, l)| (i, l.unwrap())) {
                if info.contains_key(&(index + 1)) {
                    if re.is_match(&line) {
                        writeln!(&mut new_content, "{}", re.replace(&line, "")).unwrap();
                    } else if re_default.is_match(&line) {
                        let m = re_default_type
                            .find(&info.get(&(index + 1)).unwrap().message.children[0].message)
                            .unwrap();
                        writeln!(
                            &mut new_content,
                            "{}",
                            re_default.replace(&line, &m.as_str()[1..m.len() - 1])
                        )
                        .unwrap();
                    } else {
                        writeln!(&mut new_content, "{}", line).unwrap();
                    }
                } else {
                    writeln!(&mut new_content, "{}", line).unwrap();
                }
            }

            fs::remove_file(&path).unwrap();
            let mut file = OpenOptions::new()
                .append(false)
                .write(true)
                .create(true)
                .open(path)
                .unwrap();

            file.write_all(&new_content).unwrap();
            file.sync_all().unwrap();
        }
    }
}
