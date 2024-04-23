import { VideoChunk, VideoInfo } from "../Interfaces";
import { useRef, useState } from "react";
import { Grid } from "@mui/material";
import { Data, StatCard, createData } from "../components/StatCard";
import VideoPlayer from "../components/VideoPlayer";
import { invoke } from "@tauri-apps/api/tauri";

function VideoPlayPage({ vidInfo }: { vidInfo: VideoInfo }) {
  const endpoint = 'http://localhost:' + vidInfo.ip.toString() + '/' + vidInfo.vid.toString() + '/playlist.m3u8';
  // const endpoint = "http://localhost:3031/2/playlist.m3u8";
  console.log(endpoint);

  const childRef = useRef();

    

  //handler for seeking
  // TODO: what does this do?
  const seek = (seconds: number) => {
    console.log(seconds);
  };

  const stats = async () => {
    // let chunks = await fetch('http://localhost:' + vidInfo.ip.toString() + '/' + vidInfo.vid.toString() + '/stats');
    let chunks = await (await fetch('http://localhost:' + vidInfo.ip.toString() + '/' + vidInfo.vid.toString() + '/stats')).json();
    console.log("chunks", chunks);
    let dataset = []
    for (var i = 0; i < chunks.length; i++) {
      let chunk = chunks[i];
      let data = createData(chunk.cid, chunk.addr, chunk.len, chunk.start, chunk.end);
      console.log(chunk);
      dataset.push(data);
    }
    childRef.current.addStats(dataset);
  }

  return (
    <Grid
      container
      spacing={2}
      sx={{ mt: 5, pt: 4, px: 5 }}
      color="primary"
      justifyContent="center"
    >
      <VideoPlayer
        url={endpoint}
        seekHandler={seek}
        readyHandler={stats}
        vidInfo={vidInfo}
      ></VideoPlayer>
      <StatCard ref={childRef} ></StatCard>
    </Grid>
  );
}

export default VideoPlayPage;
