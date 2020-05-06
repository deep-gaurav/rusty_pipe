use std::collections::hash_map::HashMap;

pub fn remove_non_digit_chars<T: std::str::FromStr>(input:&str)->Result<T,T::Err>{
    let re = regex::Regex::new("\\D+").unwrap();
    let onlydigits = re.replace_all(input,"");
    let count = onlydigits.parse::<T>();
    count
}

pub fn compat_parse_map(input:&str)->HashMap<String,String>{
    let mut map = HashMap::new();
    for arg in input.split("&"){
        let split_arg:Vec<&str> = arg.split("=").collect();
        if let Some(arg_p) = split_arg.get(0){
            map.insert(
                format!("{}",arg_p),
                String::from(
                    percent_encoding::percent_decode_str(&format!("{}",split_arg.get(1).unwrap_or(&""))).decode_utf8_lossy()
                )
            );
        }
    }
    map
}