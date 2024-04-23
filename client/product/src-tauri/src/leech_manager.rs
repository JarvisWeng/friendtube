use std::collections::HashMap;
use std::net::SocketAddr;
use http::{ Method, Request };
use std::cmp::Ordering::{Less, Equal, Greater};
// use serde_json::{ Value, Error};
use crate::shared::http_client::HttpClient;
use crate::shared::metadata::{
    ChunkId, ClusterOp, NodeVideoState, VideoChunk, VideoId
};

#[derive(Debug)]
pub struct LeechManager {
    leech_client: HttpClient,
    req_map: HashMap<String,u32>,
}

impl LeechManager {
    pub fn new() -> LeechManager {
        println!("[LM] created");
        LeechManager {
            leech_client: HttpClient::new(),
            req_map: HashMap::new(),
        }
    }

    pub fn get_node_state(&self, vid: VideoId, node_addr: SocketAddr) -> Option<NodeVideoState> {
        //build an http request
        let uri = format!("/get_node_state?videoId={}", vid);

        let ERR:&str = &format!("[LM] ERROR in get_node_state : could not build request with uri= {}",uri);
        let req = Request::builder().method(Method::GET).uri(uri).body(vec![]).expect(ERR);

        //send request
        let resp = self.leech_client.request(req, node_addr)?;
        let (_headers, body) = resp.into_parts();
        let metadata: NodeVideoState = match serde_json::from_slice(&body){
            Ok(s)=>s,
            Err(e)=>{
                println!("[LM] ERROR while deserialising body at get_node_state");
                println!("[LM] ERROR body vector = {:?}",&body);
                println!("[LM] ERROR body string = {:?}",std::str::from_utf8(&body));
                println!("[LM] ERROR = {:?}",e.to_string());
                panic!();
            }
        };
        return Some(metadata);
    }

    //gets the chunk from the least requested node
    //has a fallback list in case a request failed
    //only returns None after all cadidate nodes have been tried
    pub fn get_chunk(
        &mut self,
        cluster_op: &ClusterOp,
        vid: VideoId,
        cid: ChunkId
    ) -> Option<VideoChunk> {
        println!("LeechManager::get_chunks");
        //get video metadata from each cluster
        let mut nodes: Vec<&String> = vec![];
        // TODO: need to test this
        for c in cluster_op.members.iter() {
            let addr: SocketAddr = c.parse().unwrap();
            match self.get_node_state(vid, addr){
                Some(md)=>{
                    for i in &md {
                        if *i == cid {
                            nodes.push(c);
                        }
                    }
                }
                None=>{//prbably could not contact the node
                    continue;
                }
            }
        }

        // Next run some load balancing algo here
        //currently sends a request to the node that we have sent the least requests to

        if nodes.is_empty() { //none of them have the video
            return None;
        }

        //create a prority list in case we need to fallback
        nodes.sort_by(|a,b| {
            let a = match self.req_map.get(*a){
                Some(a)=>*a,
                None=>0,
            };
            let b = match self.req_map.get(*b){
                Some(a)=>*a,
                None=>0,
            };

            if a < b {
                Less
            } else if a==b {
                Equal
            } else {
                Greater
            }
        });

        //now actually send the request

        let uri: String = format!("/chunk?videoId={}&chunkId={}", vid, cid);

        let req = Request::builder()
            .method(Method::GET)
            .header(http::header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
            .uri(uri)
            .body(Vec::<u8>::new())
            .unwrap();

        

        for nd in nodes{
            if let Some(rsp) = self.leech_client.request(req.clone(),nd.parse().unwrap()){
                //update the req map
                if self.req_map.contains_key(nd){
                    *self.req_map.get_mut(nd).unwrap()+=1;
                } else {
                    self.req_map.insert(nd.clone(), 1);
                }

                let body = rsp.into_body();
                let ret = VideoChunk {
                    vid: vid, // Temp
                    cid: cid,
                    addr: nd.to_string(),
                    chunkLengthInBytes: body.len(),
                    chunkData: body.into(),
                };
        
                return Some(ret);
            }
        }

        return None;

    }
}
