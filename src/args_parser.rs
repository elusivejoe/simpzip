pub struct RunArguments {
    pub path: String
}

pub fn parse_args(args: &Vec<String>) -> Option<RunArguments> {
    match args.get(1) {
        None => {
            None
        },
        Some(val) => {
            Some(
                RunArguments {
                    path: val.to_owned()
                }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::args_parser::{parse_args};

    #[test]
    fn parse_src_file_path() {
        let args = vec!["path/to/exe".to_owned(), "L:/tests/test1.zip".to_owned()];

        match parse_args(&args) {
            None => {assert!(false, "You shouldn't be there.")},
            Some(args) => {assert_eq!(args.path, "L:/tests/test1.zip".to_owned())}
        }
    }
}