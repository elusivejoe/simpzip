pub struct RunArguments {
    pub in_file: String,
    pub out_folder: String,
}

pub fn parse_args(args: &Vec<String>) -> Result<RunArguments, &'static str> {
    let err_not_enough_params = "Not enough actual parameters.";

    let result = RunArguments {
        in_file: args.get(1).ok_or(err_not_enough_params)?.to_owned(),
        out_folder: args.get(2).ok_or(err_not_enough_params)?.to_owned(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::args::input_parser::parse_args;

    #[test]
    fn parse_src_file_path() {
        let args = vec![
            "path/to/exe".to_owned(),
            "L:/tests/test1.zip".to_owned(),
            "X:/tests/test1".to_owned(),
        ];

        match parse_args(&args) {
            Err(_) => assert!(false, "You shouldn't be there."),
            Ok(args) => {
                assert_eq!(args.in_file, "L:/tests/test1.zip".to_owned());
                assert_eq!(args.out_folder, "X:/tests/test1".to_owned());
            }
        }

        let args = vec!["path/to/exe".to_owned(), "L:/tests/test1.zip".to_owned()];

        match parse_args(&args) {
            Err(err) => assert_eq!(err, "Not enough actual parameters."),
            Ok(_) => assert!(false, "You shouldn't be there."),
        }
    }
}
