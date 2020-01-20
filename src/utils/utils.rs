

pub fn remove_non_digit_chars<T: std::str::FromStr>(input:&str)->Result<T,T::Err>{
    let re = regex::Regex::new("\\D+").unwrap();
    let onlydigits = re.replace_all(input,"");
    let count = onlydigits.parse::<T>();
    count
}