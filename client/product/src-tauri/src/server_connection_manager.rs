use std::io::Write;
use std::net::{SocketAddr, TcpStream };
use http::{ Request, Response, Method };
use local_ip_address::local_ip;
use sha2::{Sha256, Digest};
use crate::shared::http_client::read_http;
use crate::shared::http_codec::HttpCodec;
use crate::shared::metadata::{
    ChunkId, ClusterNodeMd, ClusterOp, VideoChunk, VideoDisplayMetadata, VideoId, HOST, PORT, LoginInfo
};
use dashmap::DashMap;

#[derive(Debug)]
pub struct ServerConnectionManager {
    pub server: SocketAddr,
    stream: TcpStream,
    node_md: ClusterNodeMd,
    video_map: DashMap<VideoId, VideoDisplayMetadata>,
    thumbnail_map: DashMap<VideoId, Vec<u8>>,
}

impl ServerConnectionManager {
    pub fn new(seed_port: u16) -> ServerConnectionManager {
        let addr: SocketAddr = format!("{}:{}", HOST, PORT).to_string().parse().unwrap();
        ServerConnectionManager {
            server: addr,
            stream: TcpStream::connect(addr).expect("could not connect to the server"),
            node_md: ClusterNodeMd {
                ip: local_ip().unwrap().to_string(),
                port: seed_port,
                node_id: String::new(),
            },
            video_map: DashMap::new(),
            thumbnail_map: DashMap::new()
        }
    }

    pub fn print(&self) -> () {
        println!("{:?}", self);
    }

    pub fn register(&mut self){
        // TODO: Need to open a stream instead
        println!("Running: ServerConnectionManager::{}", "connect");
        println!("server : {}", self.server.to_string());
        //create request
        let req: Request<Vec<u8>> = Request::builder()
            .method(Method::GET)
            .uri(format!("/register/?ip={}&port={}", self.node_md.ip, self.node_md.port))
            //.header(http::header::HOST,self.server.to_string())
            //.header("Connection", "close")
            .body(vec![])
            .unwrap();

        //encode request to be send over
        // println!("Request {:?}", req);
        let response = self.send_request(req);
        // println!("Response: {:#?}", response.body());

        let input_data: String = String::from_utf8(response.body().to_vec()).expect("Found invalid UTF-8");

        self.node_md.node_id = input_data;
        println!("Node Id: {}", self.node_md.node_id);

        println!("Connected Successfully!\n");
    }

    pub fn get_video_md(&mut self, video_id: VideoId) -> VideoDisplayMetadata {
        println!("Running: ServerConnectionManager::{}", "get_video_md");
        // TODO: Do a search from "video_name" to video_id

        // TODO: send own internal list of VideoMd back to server
        if !self.video_map.contains_key(&video_id) {
            println!("fetching: {}", video_id);
            let endpoint: String = format!(
                "http://{}:{}/metadata/?videoId={}&id={}/",
                HOST,
                PORT,
                video_id,
                self.node_md.node_id
            );
            println!("endpoint: {}", endpoint);

            let req = Request::builder()
                .method(Method::GET)
                .header(http::header::CONTENT_TYPE, "application/json")
                .uri(endpoint)
                // .body(serde_json::to_vec(&self.node_md).unwrap() )
                .body(vec![])
                .unwrap();

            println!("Request {:?}", req);

            let response = self.send_request(req);
            println!("Response: {}", response.status());

            let input_data: VideoDisplayMetadata = serde_json
                ::from_str(std::str::from_utf8(response.body()).unwrap())
                .unwrap();


            self.video_map.insert(input_data.vid, input_data);

            println!("Done!\n");
        }

        return self.video_map.get(&video_id).unwrap().clone();
    }

    pub fn get_video_thumbnail(&mut self, video_id: VideoId) -> Vec<u8> {
        // TODO: send own internal list of VideoMd back to server
        if !self.thumbnail_map.contains_key(&video_id) {
            println!("fetching: {}", video_id);
            let endpoint: String = format!(
                "http://{}:{}/thumbnails/?videoId={}",
                HOST,
                PORT,
                video_id
            );

            println!("endpoint: {}", endpoint);

            let req = Request::builder()
                .method(Method::GET)
                .uri(endpoint)
                // .body(serde_json::to_vec(&self.node_md).unwrap() )
                .body(vec![])
                .unwrap();

            println!("Request {:?}", req);

            let response = self.send_request(req);
            println!("Response: {}", response.status());


            self.thumbnail_map.insert(video_id, response.into_body());

            println!("Done!\n");
        }

        return self.thumbnail_map.get(&video_id).unwrap().clone();

        
    }

    pub fn login(&mut self, email: &String, pwd: &String) -> Option<Vec<u8>> {
        println!("Running: ServerConnectionManager::{}", "login");
        
        //send email
        let endpoint: String = format!(
            "http://{}:{}/login/?option=login&email={}",
            HOST,
            PORT,
            email
        );

        let req = Request::builder()
            .method(Method::GET)
            .uri(endpoint)
            // .body(serde_json::to_vec(&self.node_md).unwrap() )
            .body(vec![])
            .unwrap();

        println!("Request {:?}", req);

        let response = self.send_request(req);

        if response.status() == http::status::StatusCode::NOT_FOUND{
            return None
        }
        
        println!("Response: {}", response.status());
        //get salt and hash
        let info: LoginInfo = serde_json
        ::from_str(std::str::from_utf8(response.body()).unwrap())
        .unwrap();

        println!("{:?}",info);

        //send back Hash(Hash(pw||salt)||ch)
        let mut hasher1 = Sha256::new();
        let mut hasher2 = Sha256::new();
        let mut s:Vec<u8> = pwd.to_string().as_bytes().to_vec();
        s.append(&mut info.salt.to_string().as_bytes().to_vec());
        hasher1.update(s);
        let mut hash:Vec<u8> = hasher1.finalize()[..].to_owned();
        hash.append(&mut info.challenge.to_string().as_bytes().to_vec());
        hasher2.update(hash);
        let hash = hasher2.finalize();
        self.stream.write(&hash);

        let resp = read_http(&mut self.stream).unwrap();
        let response: Response<Vec<u8>> = HttpCodec::decode_response(resp);
        println!("{:?}",response);
        if !response.status().is_success(){
            None
        } else {
            Some(response.into_body())
        }

    }

    pub fn signup(&mut self, email: &String,  uname: &String, pwd: &String, conf: &String) -> bool {
        println!("Running: ServerConnectionManager::{}", "login");
        
        //send email
        let endpoint: String = format!(
            "http://{}:{}/login/?option=signup&email={}&username={}",
            HOST,
            PORT,
            email,
            uname
        );

        let req = Request::builder()
            .method(Method::GET)
            .uri(endpoint)
            // .body(serde_json::to_vec(&self.node_md).unwrap() )
            .body(vec![])
            .unwrap();

        println!("Request {:?}", req);

        let response = self.send_request(req);
        println!("Response: {:?}", response);
        //get salt
        let salt: String = std::str::from_utf8(response.body()).unwrap().to_string();

        println!("{:?}",salt);

        //send back Hash(pw||salt)
        let mut hasher = Sha256::new();
        let mut s:Vec<u8> = pwd.to_string().as_bytes().to_vec();
        s.append(&mut salt.to_string().as_bytes().to_vec());
        hasher.update(s);
        let hash = hasher.finalize();
        self.stream.write(&hash);

        let resp = read_http(&mut self.stream);

        println!("{:?}",std::str::from_utf8(&resp.unwrap()));

        true
    }

    pub fn get_playlist(&mut self, vid:VideoId)->Vec<u8>{
        println!("get_playlist");
        let mut ret = vec![];

        let endpoint: String = format!(
            "/manifest/?videoId={}",
            vid
        );

        let req = Request::builder()
            .method(Method::GET)
            .header(http::header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
            .uri(endpoint)
            .body(Vec::<u8>::new())
            .unwrap();

        let response = self.send_request(req);

        ret = response.into_body();

        ret
    }

    pub fn get_chunk(&mut self, vid:VideoId, cid:ChunkId)->VideoChunk{
        let mut ret = vec![];

        let endpoint: String = format!(
            "/chunk/?videoId={}&chunkId={}",
            vid,
            cid
        );

        let req = Request::builder()
            .method(Method::GET)
            .header(http::header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
            .uri(endpoint)
            .body(Vec::<u8>::new())
            .unwrap();

        let response = self.send_request(req);

        ret = response.into_body();

        let chunk = VideoChunk {
            vid: vid,
            cid: cid,
            addr: "server".to_string(),
            chunkLengthInBytes: ret.len(),
            chunkData: ret
        };
        chunk
    }

    pub fn join_cluster(&mut self, video_id: VideoId) -> ClusterOp {
        println!("Running: ServerConnectionManager::{}", "join_cluster");

        // TODO: Do a search from "video_name" to video_id

        let endpoint: String = format!(
            "http://{}:{}/cluster/?add={}&nodeId={}",
            HOST,
            PORT,
            video_id,
            self.node_md.node_id
        );

        let req = Request::builder()
            .method(Method::GET)
            .header(http::header::CONTENT_TYPE, "application/json")
            .uri(endpoint)
            .body(serde_json::to_vec(&self.node_md).unwrap())
            .unwrap();

        let response = self.send_request(req);
        let input_data: Vec<ClusterOp> = serde_json::from_slice(response.body()).unwrap();

        println!("Done!\n");

        return input_data[0].clone();
    }

    fn send_request(&mut self, req: Request<Vec<u8>>) -> Response<Vec<u8>> {
        let request = HttpCodec::encode_request(req);

        // println!("writing");
        let _ = self.stream.write_all(&request);

        //receive response
        let mut rsp: Vec<u8> = read_http(&mut self.stream).unwrap(); // I think we should panic if the server fails

        //decode response
        let response: Response<Vec<u8>> = HttpCodec::decode_response(rsp);
        // println!("return");
        return response;
    }
}
