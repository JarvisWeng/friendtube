
use std::collections::HashMap;
use std::sync::Arc;
use std::net::{ IpAddr, SocketAddr};
use crate::buffer_manager::BufferManager;
use crate::leech_manager::LeechManager;
use crate::server_connection_manager::ServerConnectionManager;
use crate::shared::metadata::{ ChunkId, VideoId, Stats };
use crate::shared::http_server::HttpServer;
use std::time::{ SystemTime, UNIX_EPOCH };
use sha2::{Sha256, Digest};

pub struct VideoServer {
    socket: SocketAddr,
    bm: Arc<BufferManager>,
    sm: ServerConnectionManager,
    lm: LeechManager,
    stat_table: HashMap<VideoId, Vec<Stats>>, // Addr, Len, Start, End
}

impl VideoServer {
    pub fn new(my_ip: IpAddr, bm: Arc<BufferManager>, port: u16, seed_port: u16) -> VideoServer {
        println!("Server listening on : {}:{}", my_ip, port);
        let mut s = ServerConnectionManager::new(seed_port);
        s.register();
        VideoServer {
            socket: (my_ip, port).into(),
            bm: bm,
            sm: s,
            lm: LeechManager::new(),
            stat_table: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        println!("starting video server");
        let mut server = HttpServer::new(self.socket);

        server.run(|req|{
            let start = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
            println!("path : {}", req.uri().path());
            match (req.method(), req.uri().path()) {
                // TODO: get a list instead
                (&http::Method::GET, "/get_video_md") => {
                    let params: HashMap<String, String> = req
                        .uri()
                        .query()
                        .map(|v| { url::form_urlencoded::parse(v.as_bytes()).into_owned().collect() })
                        .unwrap_or_else(HashMap::new);
    
                    let video_id = params.get("videoId").unwrap().parse().unwrap();
                    
                    let md = self.sm.get_video_md(video_id);
                    let body = serde_json::to_vec(&md).unwrap();

                    let rsp = http::Response
                        ::builder()
                        .status(http::StatusCode::OK)
                        .header(http::header::CONTENT_TYPE, "application/json")
                        .header(http::header::CONTENT_LENGTH, body.len())
                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(body)
                        .unwrap();
                    rsp
                }
                (&http::Method::POST, "/login") => {
                    let params: HashMap<String, String> = req
                        .uri()
                        .query()
                        .map(|v| { url::form_urlencoded::parse(v.as_bytes()).into_owned().collect() })
                        .unwrap_or_else(HashMap::new);
    
                    let email: &String = params.get("email").unwrap();
                    let password: &String = params.get("password").unwrap();
    
                    let login = self.sm.login(email,password);

                    match login{
                        None=>{
                            let rsp = http::Response
                            ::builder()
                            .status(http::StatusCode::NOT_FOUND)
                            .header(http::header::CONTENT_TYPE, "application/json")
                            .header(http::header::CONTENT_LENGTH, 0)
                            .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                            .body(vec![])
                            .unwrap();
                            rsp
                        },
                        Some(r)=>{
                            let rsp = http::Response
                                ::builder()
                                .status(http::StatusCode::OK)
                                .header(http::header::CONTENT_TYPE, "application/json")
                                .header(http::header::CONTENT_LENGTH, r.len())
                                .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                .body(r)
                                .unwrap();
                            rsp
                        }
                    }


                }

                (&http::Method::POST, "/signup") => {
                    let params: HashMap<String, String> = req
                        .uri()
                        .query()
                        .map(|v| { url::form_urlencoded::parse(v.as_bytes()).into_owned().collect() })
                        .unwrap_or_else(HashMap::new);
    
                    let username: &String = params.get("username").unwrap();
                    let password: &String = params.get("password").unwrap();
                    let email: &String = params.get("email").unwrap();
                    let confirmation: &String = params.get("confirmation").unwrap();
    
                    let md = self.sm.signup(email,username,password,confirmation);
                    let body = serde_json::to_vec(&md).unwrap();
    
                    let rsp = http::Response
                        ::builder()
                        .status(http::StatusCode::OK)
                        .header(http::header::CONTENT_TYPE, "application/json")
                        .header(http::header::CONTENT_LENGTH, body.len())
                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(body)
                        .unwrap();
                    rsp
                }

                (&http::Method::GET, "/thumbnail") => {
                    let params: HashMap<String, String> = req
                        .uri()
                        .query()
                        .map(|v| { url::form_urlencoded::parse(v.as_bytes()).into_owned().collect() })
                        .unwrap_or_else(HashMap::new);
    
                        let video_id = params.get("videoId").unwrap().parse().unwrap();
                    
                        let body = self.sm.get_video_thumbnail(video_id);

                        let rsp = http::Response
                            ::builder()
                            .status(http::StatusCode::OK)
                            .header(http::header::CONTENT_TYPE, "application/json")
                            .header(http::header::CONTENT_LENGTH, body.len())
                            .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                            .body(body)
                            .unwrap();
                        rsp
                }
    
                // path comes /<videoId>/<playlist.m3u8 | chunk.ts | stats>
                (method, path) => {
                    println!("Got request");
                    // TODO: wrap this in time start and end time
                    let parts = path.split('/').collect::<Vec<&str>>();
                    println!("{:?}", parts);
                    let video_id = parts[1].parse::<VideoId>().unwrap();
                    let file = parts[2];
    
                    match file {
                        "stats" => {
                            println!("Stats");
                            if self.stat_table.contains_key(&video_id) {
                                let stats = self.stat_table.get(&video_id).unwrap().clone();
                                // println!("{:?}", stats);
                                let stats = serde_json::to_vec(&stats).unwrap();
                                let rsp = http::Response
                                    ::builder()
                                    .status(http::StatusCode::OK)
                                    .header(http::header::CONTENT_TYPE, "application/json")
                                    .header(http::header::CONTENT_LENGTH, stats.len())
                                    .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                    .body(stats)
                                    .unwrap();
                                rsp
                            } else {
                                let rsp = http::Response
                                    ::builder()
                                    .status(http::StatusCode::OK)
                                    .header(http::header::CONTENT_TYPE, "application/json")
                                    .header(http::header::CONTENT_LENGTH, 0)
                                    .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                    .body(vec![])
                                    .unwrap();
                                rsp
                            }
                        }
                        _ => {
                            let f = file.split('.').collect::<Vec<&str>>();
                            let name = f[0];
                            let ext = f[1];
    
                            match ext {
                                "m3u8" => {
                                    // TODO: create this function
                                    // println!("I am here");
                                    let manifest = self.sm.get_playlist(video_id);
                                    let end = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards");
    
                                    self.add_stats(
                                        video_id,
                                        0, // TODO: Setting manifest as chunk 0
                                        "server".to_string(),
                                        manifest.len(),
                                        start.as_millis(),
                                        end.as_millis()
                                    );
    
                                    let rsp = http::Response
                                        ::builder()
                                        .status(http::StatusCode::OK)
                                        .header(
                                            http::header::CONTENT_TYPE,
                                            "application/vnd.apple.mpegurl"
                                        )
                                        .header(http::header::CONTENT_LENGTH, manifest.len())
                                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                        .body(manifest)
                                        .unwrap();
                                    rsp
                                }
                                "ts" => {
                                    // Check if BM has chunk first
                                    // println!("{}", name[8..].to_string());
                                    let chunk_id = name[8..].parse::<ChunkId>().unwrap();
                                    let chunk = match self.bm.get_chunk(video_id, chunk_id) {
                                        Some(chunk) => chunk,
                                        None => {
                                            // TODO: check if errors occur
                                            // TODO: either do join/check cluster for info. Cluster info should be kept in the server_manager?
                                            let cluster_op = self.sm.join_cluster(video_id);
    
                                            // Get rest of chunks from leech manager
                                            // let c = self.sm.get_chunk(video_id, chunk_id);
                                            let c = match self.lm.get_chunk(&cluster_op, video_id, chunk_id){
                                                None=>{
                                                    self.sm.get_chunk(video_id, chunk_id)
                                                },
                                                Some(l)=>{
                                                    //perform verification
                                                    let md = self.sm.get_video_md(video_id);
                                                    let mut hasher = Sha256::new();
                                                    hasher.update(&l.chunkData);
                                                    let hash:Vec<u8> = hasher.finalize().to_vec();
                                                    if md.hashes[l.cid as usize] == hex::encode(hash){
                                                        l
                                                    } else {
                                                        self.sm.get_chunk(video_id, chunk_id)
                                                    }

                                                }
                                            };
                                            // TODO: add leech chunk into the buffer manager
                                            let _ = self.bm.add_chunk(c.clone());
                                            c
                                        }
                                    };
    
                                    let end = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .expect("Time went backwards");
    
                                    self.add_stats(
                                        video_id,
                                        chunk_id + 1, // TODO: +1 for manifest???
                                        chunk.addr,
                                        chunk.chunkLengthInBytes,
                                        start.as_millis(),
                                        end.as_millis()
                                    );
    
                                    let rsp = http::Response
                                        ::builder()
                                        .status(http::StatusCode::OK)
                                        .header(http::header::CONTENT_TYPE, "video/mp2t")
                                        .header(http::header::CONTENT_LENGTH, chunk.chunkLengthInBytes)
                                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                        .body(chunk.chunkData)
                                        .unwrap();
                                    rsp
                                }
                                _ => {
                                    let rsp = http::Response
                                        ::builder()
                                        .status(http::StatusCode::NOT_FOUND)
                                        .header(http::header::CONTENT_LENGTH, 0)
                                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                        .body(vec![])
                                        .unwrap();
                                    rsp
                                }
                            }
                        }
                    }
                }
            }
        });
        
    }

    fn add_stats(&mut self, video_id: VideoId, cid: ChunkId, addr: String, len: usize, start: u128, end: u128) {
        if !self.stat_table.contains_key(&video_id) {
            self.stat_table.insert(video_id, vec![]);
        }
        let stats = self.stat_table.get_mut(&video_id).unwrap();
        let stat: Stats = Stats {
            cid: cid,
            addr: addr,
            len: len,
            start: start,
            end: end,
        };
        stats.push(stat);
    }
}
