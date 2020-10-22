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
                let path = &uri[path_pos.saturating_add(1)..];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_valid_uri() {
        let result = ParsedUrl::new("http://www.example.com/this/is/path");
        let actual = Some(
            ParsedUrl {
                host: String::from("www.example.com"),
                path: String::from("this/is/path"),
            }
        );
        assert_eq!(actual, result);
    }

    #[test]
    fn test_can_return_none_with_invalid_uri() {
        let result = ParsedUrl::new("thisisinvalidurl.com");
        assert!(result.is_none());
    }
}
