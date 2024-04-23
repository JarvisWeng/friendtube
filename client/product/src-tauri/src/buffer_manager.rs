use std::collections::HashMap;

use dashmap::DashMap;
use crate::shared::metadata::{
    ChunkId,
    VideoId,
    VideoChunk,
    VideoDisplayMetadata
};

///// Seed Manager Imports  /////

pub struct BufferManager {
    videos: DashMap<(VideoId,ChunkId), VideoChunk>, // these are the 'cached' videos
}

impl BufferManager {
    pub fn new() -> Self {
        BufferManager {
            videos: DashMap::new(),
        }
    }

    pub fn get_chunk(&self, vid: VideoId, cid: ChunkId) -> Option<VideoChunk> {
        // clone the cids argument until we figure out if we're allowed to work on cids directly
            // ToDo: figure out if we can remove this line
            println!("get_chunk: received request for chunk: {}", cid);
    
            // check if chunk is stored locally
            match self.videos.get(&(vid,cid)) {
                None => {
                    return None;
                }
                Some(chunk) => {
                    // creates an owned copy of the borrowed chunk
                    return Some(chunk.to_owned());
                }
            }
    }

    // returns a vector of chunks that are stored locally
    pub fn get_state(&self, vid: VideoId) -> Vec<ChunkId> {

        // go through every chunk in the buffer manager, and check if it's vido id matches the one we're looking for
        let mut chunks: Vec<ChunkId> = vec![];
        
        for entry in self.videos.iter() {
            // let entry: RefMulti<'_, (u64, u64), VideoChunk>
            let (video_id, chunk_id) = entry.key();
            if *video_id == vid {
                chunks.push(*chunk_id);
            }
        }

        chunks

    }

    // function to add a video chunk to the buffer manager
    pub fn add_chunk(&self, chunk: VideoChunk) -> Result<(), String> {
        // check if the chunk is already stored
        match self.videos.get(&(chunk.vid,chunk.cid)) {
            None => {
                self.videos.insert((chunk.vid,chunk.cid), chunk);
                return Ok(());
            }
            Some(_) => {
                return Err("Chunk already exists".to_string());
            }
        }
    }

    // pub fn get_video_md(&mut self, video_id: VideoId){
    //     let md = self.server.get_video_md(video_id);
    //     return md.clone();
    // }

    // async fn notify_CM_added(&self, video_md: &VideoMd, chunk_nums: &Vec<ChunkId>) -> Result<bool, String>{
    //     unimplemented!()
    // }

    // async fn notify_CM_dropped(&self, video_md: &VideoMd, chunk_nums: &Vec<ChunkId>) -> Result<bool, String>{
    //     unimplemented!()
    // }

    fn join_cluster(&self, video_id: &VideoId) -> Result<bool, String> {
        unimplemented!()

        // a node will come in with a request for a certain video chunk

        // we will check if we have this video chunkß
            // if we do, we will check if the node is in our cluster
    }

    // async fn order_chunks(chunks: &mut LinkedList<VideoChunk>) -> Result<(), String>{
    //     unimplemented!()
    //     // sort the VideoChunks in the linked list in order of VideoChunk.cid
    // }

    // fn check_cluster(&self, node_id: &str, cluster: &Vec<ClusterNodeMd>) -> Result<bool, String> {
    //     unimplemented!()
    // }

    // ToDo: need to implement seed manager somehow
}

