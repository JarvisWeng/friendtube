#![allow(non_snake_case)]

use serde::{ Deserialize, Serialize };
use std::clone::Clone;
// use serde_json::{Result, Value};
// use std::{collections::LinkedList, vec};

// TODO: put backend link
pub const HOST: &str = "192.168.43.82";
pub const PORT: i32 = 3000;

pub type VideoId = u64;
pub type ChunkId = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginInfo {
    pub salt: String,
    pub challenge: String,
}

// Describes a particular node in a Cluster
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusterNodeMd {
    pub ip: String,
    pub port: u16,
    pub node_id: String,
}

// Cluster Metadata for a Video
// Describes which nodes hold the video
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoMd {
    pub videoId: VideoId,
    pub numChunks: ChunkId,
    pub clusterList: Vec<String>,
}

// Describes info about a Video
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoDisplayMetadata {
    pub vid: VideoId,
    pub title: Box<str>,
    pub description: Box<str>,
    pub uploadDate: Box<str>,
    pub views: u64,
    pub likes: u64,
    pub dislikes: u64,
    pub creator: Box<str>,
    pub videoLength: u64,
    pub numChunks: u64,
    pub hashes: Vec<String>,
}

// Gets Node Metadata about a Video
// Which video has how many chunks
pub type NodeVideoState = Vec<ChunkId>;

// Describes a Video Chunk
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoChunk {
    pub vid: VideoId,
    pub cid: ChunkId,
    pub addr: String,

    pub chunkLengthInBytes: usize,

    pub chunkData: Vec<u8>,
}

// Describes a complete Video
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Video {
    metadata: VideoMd,
    leChunks: Vec<VideoChunk>,
}

// Describes a cluster info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusterOp {
    pub vid: VideoId,
    pub operation: Box<String>,
    pub members: Vec<String>,
    pub timestamps: Vec<f32>,
}

// Describes a cluster info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stats {
    pub addr: String,
    pub cid: ChunkId,
    pub len: usize,
    pub start: u128,
    pub end: u128,
}
