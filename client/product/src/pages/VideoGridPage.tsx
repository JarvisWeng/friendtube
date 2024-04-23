import { Grid, Container } from "@mui/material";
import VideoCard from "../components/VideoCard";
import { useRef, useEffect, useState, useCallback } from "react";
import { VideoInfo } from "../Interfaces";
import { invoke } from "@tauri-apps/api/tauri";

async function fetchVideoCards({ videoClickHandler }, vid: number) {
  let ip: number = await invoke("get_port");
  // console.log(ip);
  let resp = await fetch('http://localhost:' + ip.toString() + '/get_video_md?videoId=' + vid.toString());
  let thumbnail = await fetch('http://localhost:' + ip.toString() + '/thumbnail?videoId=' + vid.toString())
  let md: VideoInfo = {} as VideoInfo;
  Object.assign(md, await resp.json());
  md.ip = ip;
  console.log("md", md);
  return [
    <VideoCard key={vid} videoClickHandler={videoClickHandler} info={md} thumbnail={URL.createObjectURL(await thumbnail.blob())}></VideoCard>
  ];
}

function VideoGridPage({ videoClickHandler }) {
  const [videoCards, setVideoCards] = useState(Array(0));
  const [videoPlayHandler, setHandler] = useState({ videoClickHandler });
  const observerTarget = useRef(null);
  // console.log(videoPlayHandler);
  const scrollEffect = () => {
    let observerCallback = (entries, observer) => {
      if (entries[0].isIntersecting) {
        entries.forEach(async (entry) => {
          // console.log(videoPlayHandler);
          const data = await fetchVideoCards(videoPlayHandler, 2); // TODO: Get different video
          setVideoCards((prev) => [...prev, ...data]);
        });
      }
    };

    let options = {
      root: document.querySelector("#scrollArea"),
      rootMargin: "0px",
      threshold: 0.25,
    };

    let observer = new IntersectionObserver(observerCallback, options);
    if (observerTarget.current) {
      observer.observe(observerTarget.current);
    }
  };

  // Loading default videos
  useEffect(() => {
    for (let i = 1; i < 11; ++i) {
      let data = fetchVideoCards(videoPlayHandler, i);
      data.then((data) => {
        // Need to load > 12 in full screen
        setVideoCards((prev) => [...prev, ...data]);
      })
    }
  }, []);
  useEffect(scrollEffect, [observerTarget]);

  return (
    <Grid
      container
      spacing={2}
      sx={{ mt: 5, pt: 4, px: 5 }}
      color="primary"
      justifyContent="center"
    >
      {/* <VideoCard videoClickHandler={videoClickHandler} info={vidInfo}></VideoCard> */}
      {videoCards}
      <Grid item ref={observerTarget}></Grid>
    </Grid>
  );
}

export default VideoGridPage;

