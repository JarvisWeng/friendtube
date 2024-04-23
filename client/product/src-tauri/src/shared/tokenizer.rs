use std::collections::{HashMap, LinkedList};
use mio::net::TcpStream;

pub struct Tokenizer{
    map:HashMap<usize,Connection>,
    counter: usize,
}

impl Tokenizer{

    pub fn new()->Tokenizer{
        Tokenizer{
            map:HashMap::new(),
            counter:0,
        }
    }

    pub fn add(&mut self,stream:TcpStream, keep_alive:bool)->usize{
        self.counter+=1;
        self.map.insert(self.counter, Connection::new(stream,keep_alive));
        println!("generated Token : {} for {:?}",self.counter,self.map.get(&self.counter).unwrap().stream.peer_addr());
        self.counter
    }

    pub fn get(&self,t:usize)->Option<&Connection>{
        self.map.get(&t)
    }

    pub fn get_mut(&mut self,t:usize)->Option<&mut Connection>{
        self.map.get_mut(&t)
    }

    pub fn remove(&mut self,t:usize)-> Option<Connection>{
        let  c= self.map.remove(&t);
        match c {
            None=>None,
            Some(s) => {
                println!("removed Token : {} for {:?}",t,s.stream.peer_addr());
                Some(s)
            },
        }
    }
}
#[derive(Clone, Copy)]
pub enum State{
    EMPTY,
    FILLED,
    PARTIAL_FILLED,
    PARTIAL,
    DEAD,//DEAD state deregister
}
pub struct Connection{
    pub stream: TcpStream,
    pub pbuf: Vec<u8>,//buffer for partial reads
    pub responses: LinkedList<Vec<u8>>, //list for responses
    pub keep_alive: bool,
    pub state:State,
}

impl Connection{
    pub fn new(s:TcpStream, k:bool)->Connection{
        Connection{
            stream:s,
            pbuf:vec![],
            responses:LinkedList::new(),
            keep_alive:k,
            state:State::EMPTY,
        }
    }
}