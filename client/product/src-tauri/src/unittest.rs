use crate::video_server::VideoServer;
use local_ip_address::local_ip;

#[cfg(test)]
mod tests{
    #[test]
    fn test_video_server(){
        const VS_PORT:u16 = 3030;

        let bm = Arc::new(buffer_manager::BufferManager::new());

        //start video server - > needs leech manager for getting videos from peer
        //                   - > needs server connection manager for getting videos from server
        //                   - > needs buffer manager to get videos from local cache
        let t = bm.clone();
        thread::spawn(move || {
            let addr = "127.0.0.1".parse::<IpAddr>().unwrap();
            let mut vs = video_server::VideoServer::new(addr, t, VS_PORT);
            vs.run();
        });
    }
}

