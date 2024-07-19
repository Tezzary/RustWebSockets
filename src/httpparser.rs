pub fn get_header_value(headers: &str, key: &str) -> Result<String, String> {
    let lines = headers.lines();
    let mut value = Err("Header not found".to_string());
    for line in lines {
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split(": ").collect();
            value = Ok(parts[1].to_string());
        }
    }
    value
}