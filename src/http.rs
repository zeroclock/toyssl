// TODO: Implement UrlTrait and apply to these structs.
// It has a function to make a HTTP request header.
// In the http get method, we can make a HTTP request header transparently.

use std::net::TcpStream;
use std::{
    io::{
        prelude::*,
        BufReader,
        BufWriter,
    },
};

use anyhow::{
    Result,
    Error,
    anyhow,
};

const HTTP_PORT: u32 = 80;

/// Host and path parsed from an uri
#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub host: String,
    pub path: String,
}

impl ParsedUrl {
    /// Returns a parsed url from given uri
    pub fn new(uri: &str) -> Option<Self> {
        let host_start_pos = uri.find("//");
        if let Some(host_pos) = host_start_pos {
            let host_pos = host_pos.saturating_add(2);
            let host_and_path = &uri[host_pos..];
            let path_start_pos = host_and_path.find("/");
            if let Some(path_pos) = path_start_pos {
                let path_pos = path_pos.saturating_add(host_pos);
                let host = &uri[host_pos..path_pos];
                let path = &uri[path_pos..];
                return Some(
                    ParsedUrl {
                        host: String::from(host),
                        path: String::from(path),
                    }
                );
            }
        }
        None
    }
}

#[derive(Debug, PartialEq)]
pub struct ParsedProxyUrl {
    pub host: String,
    pub port: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ParsedProxyUrl {
    /// Returns a parsed proxy url from given uri
    /// It's forgiven not to start with 'http://'
    /// uri format: http://[username:password@]hostname[:port]/
    pub fn new(uri: &str) -> Result<Self, Error> {
        let mut host = String::new();
        let mut port = HTTP_PORT.to_string();
        let mut username: Option<String> = None;
        let mut password: Option<String> = None;
        // skipping 'http://'
        let protocol_pos = uri.find("http://");
        let mut uri = if let Some(pos) = protocol_pos {
            &uri[pos.saturating_add(7)..]
        } else {
            &uri
        };
        // login info parsing
        let login_info_pos = uri.find("@");
        if let Some(pos) = login_info_pos {
            let login_info = &uri[..pos];
            let username_pos = login_info.find(":");
            if let None = username_pos {
                // Error - malformed login ifo
                return Err(anyhow!("Supplied login info is malformed: {}", login_info));
            }
            let username_pos = username_pos.unwrap();
            if username_pos == 0 {
                // Error - if login info supplied, username must be supplied
                return Err(anyhow!("Expected username in {}", login_info));
            }
            if login_info.len().saturating_sub(1) == username_pos {
                // Error - if username supplied, password must be supplied
                return Err(anyhow!("Expected password in {}", login_info));
            }
            username = Some(String::from(&login_info[..username_pos]));
            password = Some(String::from(&login_info[username_pos.saturating_add(1)..]));
            
            uri = &uri[pos.saturating_add(1)..];
        }
        // truncate '/' at the end of uri
        let slash_pos = uri.find("/");
        if let Some(pos) = slash_pos {
            uri = &uri[..pos];
        }
        // port parsing
        let colon_pos = uri.find(":");
        if let Some(pos) = colon_pos {
            if pos == uri.len().saturating_sub(1) {
                // Error - if colon supplied, port must be supplied
                return Err(anyhow!("Expected port: {}", uri));
            }
            let p = &uri[pos.saturating_add(1)..];
            if p == "0" {
                // Error - 0 is not a valid port
                return Err(anyhow!("Port 0 is not a valid port: {}", uri));
            }
            host = format!("{}", &uri[..pos]);
            port = format!("{}", p);
        } else {
            host = format!("{}", uri);
        }
        Ok(Self {
            host,
            port,
            username,
            password,
        })
    }
}

pub fn http_get(tcp_stream: &TcpStream, parsed_url: ParsedUrl, parsed_proxy_url: Option<ParsedProxyUrl>) {
    println!("Retrieving document: '{}'", parsed_url.path);
    let mut reader = BufReader::new(tcp_stream);
    let mut writer = BufWriter::new(tcp_stream);
    
    // format HTTP request
    let header_part = if parsed_proxy_url.is_some() {
        format!("GET http://{}{} HTTP/1.1\r\n", parsed_url.host, parsed_url.path)
    } else {
        format!("GET {} HTTP/1.1\r\n", parsed_url.path)
    };
    let header = format!("{}Host: {}\r\nConnection: close\r\n\r\n", header_part, parsed_url.host);
    println!("GET request sending...");
    println!("-- Request --\n{}", header);

    tcp_write(&mut writer, &header);
    tcp_read(&mut reader);
}

fn tcp_read(reader: &mut BufReader<&TcpStream>) {
    let mut msg = String::new();
    // reader.read_line(&mut msg).expect("Failed to read lines from tcp stream");
    reader.read_to_string(&mut msg).expect("Failed to read lines from tcp stream");
    println!("{}", msg);
}

fn tcp_write(writer: &mut BufWriter<&TcpStream>, msg: &str) {
    writer.write(msg.as_bytes()).expect("Failed to send message to tcp stream");
    writer.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_valid_uri() {
        let result = ParsedUrl::new("http://www.example.com/this/is/path");
        let actual = Some(
            ParsedUrl {
                host: String::from("www.example.com"),
                path: String::from("/this/is/path"),
            }
        );
        assert_eq!(actual, result);
    }

    #[test]
    fn test_can_return_none_with_invalid_uri() {
        let result = ParsedUrl::new("thisisinvalidurl.com");
        assert!(result.is_none());
    }

    #[test]
    fn test_can_parse_valid_full_proxy_uri() {
        let result = ParsedProxyUrl::new("http://username:password@hostname.com:8888/").unwrap();
        let expected = ParsedProxyUrl {
            host: String::from("hostname.com"),
            port: String::from("8888"),
            username: Some(String::from("username")),
            password: Some(String::from("password")),
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_can_parse_valid_proxy_uri_without_some_part() {
        let result = ParsedProxyUrl::new("username:password@hostname.com").unwrap();
        let expected = ParsedProxyUrl {
            host: String::from("hostname.com"),
            port: String::from("80"),
            username: Some(String::from("username")),
            password: Some(String::from("password")),
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_can_return_error_malformed_login_info() {
        let result = ParsedProxyUrl::new("http://invalidlogininfo@hostname.com:8888");
        let expected_err_msg = "Supplied login info is malformed";
        assert!(result.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains(expected_err_msg), format!("error message should contain: {}, but actual is: {}", expected_err_msg, err_msg));
    }

    #[test]
    fn test_can_return_error_username_is_not_supplied() {
        let result = ParsedProxyUrl::new("http://:password@hostname.com:8888");
        let expected_err_msg = "Expected username in";
        assert!(result.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains(expected_err_msg), format!("error message should contain: {}, but actual is: {}", expected_err_msg, err_msg));
    }

    #[test]
    fn test_can_return_error_password_is_not_supplied() {
        let result = ParsedProxyUrl::new("http://username:@hostname.com:8888");
        let expected_err_msg = "Expected password in";
        assert!(result.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains(expected_err_msg), format!("error message should contain: {}, but actual is: {}", expected_err_msg, err_msg));   
    }

    #[test]
    fn test_can_return_error_port_is_not_supplied() {
        let result = ParsedProxyUrl::new("http://username:password@hostname.com:");
        let expected_err_msg = "Expected port";
        assert!(result.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains(expected_err_msg), format!("error message should contain: {}, but actual is: {}", expected_err_msg, err_msg));
    }

    #[test]
    fn test_can_return_error_invalid_port() {
        let result = ParsedProxyUrl::new("http://username:password@hostname.com:0");
        let expected_err_msg = "Port 0 is not a valid port";
        assert!(result.is_err(), "ParsedProxyUrl should be error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains(expected_err_msg), format!("error message should contain: {}, but actual is: {}", expected_err_msg, err_msg));
    }
}
