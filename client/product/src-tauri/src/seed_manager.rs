use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use std::net::IpAddr;
use http::{Response,Method,StatusCode};
use crate::buffer_manager::BufferManager;
use crate::shared::http_server::HttpServer;

// TODO:
// use std::path::Path;
pub struct SeedManager{
    socket : SocketAddr,
    bm : Arc<BufferManager>
}

impl SeedManager{
    pub fn new(my_ip:IpAddr, port: u16,bm : Arc<BufferManager>) -> SeedManager{
        println!("SeedManager Listening on: {}:{}", my_ip, port);
        SeedManager{
            socket:(my_ip, port).into(),
            bm:bm,
        }
    }

    pub fn run(&mut self) {
        println!("starting seed manager");
        let mut server = HttpServer::new(self.socket);
        server.run(|req|{
            match (req.method(), req.uri().path()) {
        
                // Simply echo the body back to the client.
                (&Method::GET, "/echo") => {
                    Response::new(req.into_body())
                }
    
                (&Method::GET, "/get_node_state") => {
                    let params: HashMap<String, String> = req
                        .uri()
                        .query()
                        .map(|v| {
                            url::form_urlencoded::parse(v.as_bytes())
                                .into_owned()
                                .collect()
                        })
                        .unwrap_or_else(HashMap::new);
                    
                    // TODO: user video_id
                    let video_id = params.get("videoId").unwrap();
                    println!("Requesting state for video: {}", video_id);
        
                    // TODO: user buffer manager
                    let chunk_ids = self.bm.get_state(video_id.parse().unwrap());
            
                    let resp = Response::builder()
                                .status(StatusCode::OK)
                                .header(http::header::CONTENT_TYPE, "application/json")
                                .body(serde_json::to_vec(&chunk_ids).unwrap())
                                .unwrap();
        
                    return resp;
                }
        
                (&Method::GET, "/chunk") => {
                    let params: HashMap<String, String> = req
                        .uri()
                        .query()
                        .map(|v| {
                            url::form_urlencoded::parse(v.as_bytes())
                                .into_owned()
                                .collect()
                        })
                        .unwrap_or_else(HashMap::new);
                    
                    // TODO: user video_id
                    let video_id = params.get("videoId").unwrap();
                    let chunk_id = params.get("chunkId").unwrap();
                    println!("Hi I'm called: {}/{}", video_id, chunk_id);
        
                    // TODO: user buffer manager
                    // let video = fs::read(format!("../src/assets/chunk{}.webm", chunk_id))
                    //     .expect("Should have been able to read the file");
                    let chunk = self.bm.get_chunk(video_id.parse().unwrap(), chunk_id.parse().unwrap());
    
                    // new vector of size 0
                    // if chunk is none return an error response, if some, return regular response
                    let resp = match chunk {
                        None => {
                            // response with 404
                            let resp = Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Vec::new())
                                .unwrap();
                            resp
                        }
                        Some(chunk) => {
                            let resp = Response::builder()
                                .status(StatusCode::OK)
                                .header(http::header::CONTENT_TYPE, "video/webm")
                                .body(chunk.chunkData)
                                .unwrap();
                            resp
                        }
                    };
        
                    return resp;
                }
        
                // Reverse the entire body before sending back to the client.
                //
                // Since we don't know the end yet, we can't simply stream
                // the chunks as they arrive as we did with the above uppercase endpoint.
                // So here we do `.await` on the future, waiting on concatenating the full body,
                // then afterwards the content can be reversed. Only then can we return a `Response`.
                (&Method::GET, "/echo/reversed") => {
                    let whole_body = req.into_body();
        
                    let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
                    Response::new(reversed_body)
                }
        
                // Return the 404 Not Found for other routes.
                _ => {
                    let mut not_found = Response::default();
                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                    not_found
                }
            }
        });
        
    }

}

//type StandardResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

//This is our service handler. It receives a Request, routes on its
// path, and returns a Future of a Response.


// pub async fn seed_main() -> StandardResult<()> {
//     let my_local_ip: std::net::IpAddr = local_ip().unwrap();

//     // This address is localhost
//     let addr: SocketAddr = (my_local_ip, 3030).into();

//     let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(seed)) });

//     let server = Server::bind(&addr).serve(service);

//     println!("Listening on http://{}", addr);

//     println!("This is my local IP address: {:?}", my_local_ip);

//     server.await?;

//     Ok(())
// }