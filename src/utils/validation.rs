use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_username(username: &str) -> bool {
    username.len() >= 3
        && username.len() <= 50
        && username.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn is_valid_password(password: &str) -> bool {
    password.len() >= 8
}

pub fn sanitize_string(input: &str) -> String {
    input.trim().to_string()
}
