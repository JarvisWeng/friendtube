use std::io::{ self, Read, Write };
use mio::{ Events, Interest, Poll, Token };
use mio::net::TcpListener;
use std::net::SocketAddr;
use crate::shared::http_codec::HttpCodec;
use crate::shared::tokenizer::{Tokenizer,State};



const SERVER: Token = Token(0);

pub struct HttpServer {
    socket: SocketAddr,
    tokenizer: Tokenizer,
}

impl HttpServer {
    pub fn new(addr:SocketAddr) -> HttpServer {
        println!("Server listening on : {:?}", addr);
        HttpServer {
            socket: addr,
            tokenizer:Tokenizer::new(),
        }
    }

    pub fn handle(&mut self,poll:&mut Poll,token:usize, mut registered:bool, mut rd:bool, mut wr:bool,handler:&mut impl FnMut(http::Request<Vec<u8>>) -> http::Response<Vec<u8>> ){
        let mut state = self.tokenizer.get_mut(token).unwrap().state;
        loop{
            if wr {//first write the responses
                // println!("writing");
                match state{
                    State::FILLED | State::PARTIAL_FILLED=>{
                        state = self.handle_write(token, &mut wr);
                    }
                    _ => {
                        //nothing to do turn writing off
                        // if !rd{
                        //     poll.registry().reregister(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::READABLE).expect("Unable to register");
                        //     break;
                        // }
                    }
                }
            }
            //check the state
            match state{
                State::DEAD => {
                    println!("Dead");
                    if registered {
                        poll.registry().deregister(&mut self.tokenizer.get_mut(token).unwrap().stream).expect("Unable to register");
                        self.tokenizer.remove(token);
                    }
                    break;
                },
                _=>{}
            }

            //read if possible
            if rd {
                // println!("reading");
                state = self.handle_read(token, &mut rd, handler);
            }

            //check the state and register if needed
            match state{
                State::DEAD => {
                    if registered {
                        println!("Dead");
                        poll.registry().deregister(&mut self.tokenizer.get_mut(token).unwrap().stream).expect("Unable to register");
                        self.tokenizer.remove(token);
                    }
                    break;
                }
                State::EMPTY | State::PARTIAL => {
                    wr = false;
                    if !rd {
                        if !registered {
                            poll.registry().register(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::READABLE).expect("Unable to register");
                            registered=true;
                        } else {
                            poll.registry().reregister(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::READABLE).expect("Unable to register"); 
                        }
                    }
                }
                _=>{
                    if !wr { //register for a write since state is filled
                        if !registered {
                            poll.registry().register(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::WRITABLE|Interest::READABLE).expect("Unable to register");
                            registered=true;
                        } else {
                            poll.registry().reregister(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::WRITABLE).expect("Unable to register"); 
                        }
                    }
                }
            }

            if !registered {
                poll.registry().register(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::WRITABLE|Interest::READABLE).expect("Unable to register");
                registered=true;
            }

            //exit conditions
            if !rd && !wr{
                poll.registry().reregister(&mut self.tokenizer.get_mut(token).unwrap().stream,Token(token),Interest::WRITABLE|Interest::READABLE).expect("Unable to register");
                break;
            }
        }
    }

    fn handle_read(&mut self,token:usize, rd:&mut bool, handler:&mut impl FnMut(http::Request<Vec<u8>>) -> http::Response<Vec<u8>> )->State{
        let mut buf: Vec<u8> = vec![];
        //read
        let result = self.tokenizer.get_mut(token).unwrap().stream.read_to_end(&mut buf);

        //handle the errors
        match result {
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::WouldBlock => { //new to register for poll
                        *rd = false;//will not read anymore
                    },
                    err => { //some other error
                        println!("READ ERROR: {} ",err);
                        return State::DEAD;
                    },
                };
            },
            Ok(0) => { //ok then handle the request and return the repsonse
                *rd = false;
            },
            Ok(n) => { //ok then handle the request and return the repsonse
                println!("read {} bytes",n);
            }
        };

        //checkout buf
        let mut state = self.tokenizer.get_mut(token).unwrap().state;
        if !buf.is_empty() { //rust borrowing bullshit
            // let mut s=String::new();
            // for ch in buf.iter() {
            //     s.push(*ch as char);
            // }

            // println!("req : \"{}\"", s);

            let req = HttpCodec::decode_request(buf);
            //TODO:check for Connection: keep-alive header
            let rsp = handler(req);
            // println!("{:?}", rsp);
            let response = HttpCodec::encode_response(rsp);
            let c = self.tokenizer.get_mut(token).unwrap();
            c.responses.push_back(response);
            c.state = State::FILLED;
            state = State::FILLED;
        }
        state
    }

    fn handle_write(&mut self, token: usize,wr:&mut bool)->State{
        let c = self.tokenizer.get_mut(token).unwrap();
        while let Some(buf) = c.responses.pop_front(){
            match c.stream.write(&buf){
                Err(e)=>{
                    match e.kind() {
                        io::ErrorKind::WouldBlock => { //new to register for poll
                            *wr=false;
                            c.responses.push_front(buf);
                            break;
                        }
                        err => {//some other kind of error
                            println!("WRITE ERROR: {}",err);
                            //drop the stream
                            return State::DEAD;
                        }
                    }
                }
                Ok(n)=>{//keep alive is true by default
                    if n < buf.len(){
                        c.responses.push_front(buf[n..].to_vec());
                    }
                    println!("written successfully");
                }
            }
        }
        return if c.responses.len() > 0 {c.state} else {State::EMPTY};
    }

    pub fn run(&mut self,mut handler:impl FnMut(http::Request<Vec<u8>>) -> http::Response<Vec<u8>> ) {
        println!("starting video server");
        let mut poll = Poll::new().expect("Unable to create new poll instance");
        // Create storage for events.
        let mut events = Events::with_capacity(1024);

        // Setup the server socket.
        let mut server = TcpListener::bind(self.socket).expect("Unable to bind to the socket");

        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)
            .expect("unable to register server with poll");

        loop {
            // Poll Mio for events, blocking until we get an event.
            // println!("polling");
            poll.poll(&mut events, None).expect("Error while polling events");
            // println!("EVENTS");
            // for event in events.iter(){
            //     let t = event.token().0;
            //     let c = if t > 0 {self.tokenizer.get(t).unwrap().stream.peer_addr().unwrap().to_string()} else {"server".to_string()};
            //     println!("TOKEN {} ADDR {} (WRITABLE {}) (READABLE {}) (ACCEPT {})",t,c,event.is_writable(),event.is_readable(),t==0);
            // }

            // Process each event.
            for event in events.iter() {
                // We can use the token we previously provided to `register` to
                // determine for which socket the event is.
                match event.token() {
                    SERVER => {
                        // If this is an event for the server, it means a connection
                        // is ready to be accepted.
                        while let Ok((stream, _addr)) = server.accept() {
                            println!("recieved request");
                            //create token
                            let t = self.tokenizer.add(stream,true);
                            //handle reads
                            self.handle(&mut poll, t, false, true, true, &mut handler);
                        }
                    }
                    Token(n) => {
                        match self.tokenizer.get(n){
                            Some(_)=>self.handle(&mut poll, n, true, event.is_readable(), event.is_writable(),&mut handler),
                            None=>{}//ignore
                        }
                    }
                    // We don't expect any events with tokens other than those we provided.
                    _ => unreachable!(),
                }
            }
        }
    }
}
