use std::net::{
    ToSocketAddrs,
    TcpStream,
};
use anyhow::{
    Result,
    anyhow,
    Context as _,
};

use super::http::{
    ParsedUrl,
    ParsedProxyUrl,
    http_get,
};

const HTTP_PORT: u32 = 80;

pub trait App {
    fn run(&mut self) -> Result<()>;
}

pub struct Client {
    parsed_url: ParsedUrl,
    parsed_proxy_url: Option<ParsedProxyUrl>,
}

impl Client {
    pub fn new(args: &Vec<String>) -> Result<Self> {
        // parsing url
        let mut idx: usize = 2;
        let mut parsed_proxy_url: Option<ParsedProxyUrl> = None;
        if args[idx] == "-p" {
            idx += 1;
            parsed_proxy_url = Some(ParsedProxyUrl::new(&args[idx])?);
            idx += 1;
        }
        
        let parsed_url = ParsedUrl::new(&args[idx]);

        if parsed_url.is_none() {
            return Err(anyhow!("Error - malformed URL '{}'", args[idx]));
        }

        let parsed_url = parsed_url.unwrap();

        Ok(Self {
            parsed_url,
            parsed_proxy_url,
        })
    }
}

impl App for Client {
    fn run(&mut self) -> Result<()> {
        println!("Connecting to host {}", self.parsed_url.host);

        // resolve ip from hostname
        let addrs = if let Some(proxy) = &self.parsed_proxy_url {
            format!("{}:{}", proxy.host, proxy.port).to_socket_addrs()
        } else {
            format!("{}:{}", self.parsed_url.host, HTTP_PORT).to_socket_addrs()
        };
        
        println!("Resolved IP: {:?}", addrs);

        if addrs.is_err() {
            return Err(anyhow!("Error in name resolution."));
        }

        let mut addrs = addrs.unwrap();

        if let Some(addr) = addrs.find(|x| (*x).is_ipv4()) {
            let stream = TcpStream::connect(addr).with_context(|| "Unable to connect to host.")?;
            http_get(&stream, &self.parsed_url, &self.parsed_proxy_url);
        } else {
            return Err(anyhow!("Invalid Host:Port combination."));
        }

        Ok(())
    }
}

