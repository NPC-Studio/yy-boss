mod cli {
    mod input;
    pub use input::*;
}

pub use yy_boss::*;

pub fn main() {
    let input = cli::parse_inputs();

    match YypBoss::new(&input.yyp_path) {
        Ok(_) => {
            println!("Nice job");
        }
        Err(e) => {
            eprintln!("!!Error!!: Could not load Yyp...{:?}", e);
        }
    }
}
