use crate::shared::http_codec::HttpCodec;
use http::{Request, Response};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream}
};

// reads one http req/response at a time
pub fn read_http(st: &mut TcpStream) -> Option<Vec<u8>> {
    //println!("read_http");
    let mut buffer: [u8; 1024] = [0; 1024];
    let mut reader = BufReader::new(st);
    let mut header: Vec<u8> = Vec::with_capacity(1024);
    let mut size: usize = 0;
    let mut buf:Vec<u8> = vec![];

    loop {
        // println!("before line read");
        let i = match reader.read_until(b'\n', &mut header) {
            Ok(i) => i,
            Err(e) => {
                //print some debug info
                println!("[HTTP CLIENT] ERROR in read_http while reading header lines");
                println!("[HTTP CLIENT] ERROR = {}",e.to_string());
                return None;// fatal error reached so no point trying to return an incomplete resp/req
            }
        };


        let head = match std::str::from_utf8(&header){
            Ok(h)=>h,
            Err(e)=>{
                //print some debug info
                println!("[HTTP CLIENT] ERROR in read_http while converting header line");
                //try to convert the header
                let mut s=String::new();
                for ch in header.iter() {
                    s.push(*ch as char);
                }
                println!("[HTTP CLIENT] ERROR header = {}",s);
                println!("[HTTP CLIENT] ERROR = {}",e.to_string());
                panic!(); //want to know if we are getting badly encoded bytes 
            }
        };
        // println!("header: {}", head);
        if let Some(_) = head.find("Content-Length") {
            // println!("found");
            let num: Vec<&str> = head.split(':').collect();
            //println!("{:?}", num);
            size = match num[1].trim().parse::<usize>(){
                Ok(s)=>s,
                Err(e)=>{
                    println!("[HTTP CLIENT] ERROR in read_http while parsing content length");
                    println!("[HTTP CLIENT] ERROR header = {} , Content-Length parsed = {}",head,num[1]);
                    println!("[HTTP CLIENT] ERROR = {}",e.to_string());
                    panic!(); //want to know if content length is not properly written in other parts of the code
                }
            }
        }

        buf.extend_from_slice(&header);
        if i == 2 { // find last \r\n
            break;
        }
        header.clear();
    }
    let header_len = buf.len();

    // now we know how much is left to read
    // Read until the end of the msg
    while size > 0 {
        let _ = match reader.read(&mut buffer) {
            Ok(0) => { //might reach end of file
                //should be an error since if we reach eof and size != 0;
                println!("[HTTP CLIENT] ERROR in read_http while reading content");
                let mut s=String::new();
                for ch in buf[1..header_len].iter() {
                    s.push(*ch as char);
                }
                println!("[HTTP CLIENT] ERROR request header = {}",s);
                println!("[HTTP CLIENT] ERROR = got eof buf did not recieve full request");
                panic!(); //want to know if content length is not properly written in other parts of the code
            }
            Ok(i) => {
                buf.extend_from_slice(&buffer[0..i]);
                size -= i;
            },
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::Interrupted => { //retry

                    },
                    _ => {
                        println!("[HTTP CLIENT] ERROR in read_http while parsing content length");
                        println!("[HTTP CLIENT] ERROR = {}",e.to_string());
                        return None; //return None since it can be due to network erorrs
                    }
                }
            }
        };
    }

    return Some(buf);
}

#[derive(Debug)]
pub struct HttpClient {
    streams: Vec<(usize, TcpStream, SocketAddr)>, //streams of current pending requests
}

impl HttpClient {
    pub fn new() -> HttpClient {
        HttpClient {
            streams: vec![],
        }
    }

    pub fn request(&self, request: Request<Vec<u8>>, addr: SocketAddr) -> Option<Response<Vec<u8>>> {
        let mut st: TcpStream = match TcpStream::connect(addr){
            Ok(t)=>t,
            Err(e)=>{
                println!("[HTTP CLIENT] ERROR in HttpClient::request while trying to connect to {}",addr);
                println!("[HTTP CLIENT] ERROR = {}",e.to_string());
                return None;
            }
        };

        let mut buf = HttpCodec::encode_request(request);

        match st.write_all(&buf){
            Ok(n)=>{},
            Err(e)=>{
                println!("[HTTP CLIENT] ERROR in HttpClient::request while trying to send req to {}",addr);
                println!("[HTTP CLIENT] ERROR = {}",e.to_string());
                return None;
            }
        }

        let mut response = read_http(&mut st)?;
        // println!("{:?}", response);

        let rsp = HttpCodec::decode_response(response);

        return Some(rsp);
    }
}
