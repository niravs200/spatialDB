use std::{io::BufReader, net::TcpStream, process::Child, time::Duration};
use std::io::{Write, BufRead};

use crate::constant::{HEALTH_COMMAND, HEALTH_POSITIVE_CONFIRMATION_RESPONSE, ZONE_NODE_IP};

pub struct Server {
    pub child: Child,
    pub tcp_port: u16,
    pub udp_port: u16,
}

impl Server {
    pub fn new(child: Child, tcp_port: u16, udp_port: u16) -> Self {
        Self { child, tcp_port, udp_port }
    } 

    pub fn check_health(&self) -> bool {
        if let Ok(mut stream) = TcpStream::connect((ZONE_NODE_IP, self.tcp_port)) {
            stream
                .set_read_timeout(Some(Duration::from_millis(500)))
                .ok();

            if stream.write_all(HEALTH_COMMAND).is_ok() {
                let mut reader = BufReader::new(stream);

                let mut response = String::new();

                if reader.read_line(&mut response).is_ok() {
                    return response.trim() == HEALTH_POSITIVE_CONFIRMATION_RESPONSE;
                }
            }            
        }
        false
    }
}

