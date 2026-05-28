use std::env;
use jjazz_engine::style_parser::parse_style_file;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: parse-style <file.prs|file.sty>");
        return;
    }
    match parse_style_file(&args[1]) {
        Ok(style) => {
            let json = serde_json::to_string_pretty(&style).unwrap();
            println!("{}", json);
        }
        Err(e) => eprintln!("错误: {}", e),
    }
}
