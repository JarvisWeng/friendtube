import ReactPlayer from "react-player";
import {
  Avatar,
  Button,
  ButtonGroup,
  Card,
  CardActionArea,
  CardHeader,
  Container,
  IconButton,
  Typography,
} from "@mui/material";
import ThumbUpIcon from "@mui/icons-material/ThumbUp";
import ThumbDownIcon from "@mui/icons-material/ThumbDown";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import ShareIcon from "@mui/icons-material/Share";
import { red } from "@mui/material/colors";
import { VideoInfo } from "../Interfaces";

function VideoPlayer(props: {
  url: string;
  seekHandler: (seconds: number) => void;
  readyHandler: () => void;
  vidInfo: VideoInfo;
}) {
  return (
    <Card>
      <ReactPlayer
        width={1280}
        height={720}
        url={props.url}
        controls
        // onSeek={props.seekHandler}
        // onReady={props.readyHandler}
        // onBuffer={props.readyHandler}
        // onBufferEnd={props.readyHandler}
        // onPlay={props.readyHandler}
        
        onPause={props.readyHandler}
        onEnded={props.readyHandler}
        config={{file: {forceHLS: true,}
        }}
      ></ReactPlayer>
      <Typography
        variant="h6"
        color="text.secondary"
        noWrap={true}
        sx={{ mt: 2, ml: 2 }}
      >
        {props.vidInfo.title}
      </Typography>
      <CardHeader
        avatar={
          <Avatar sx={{ bgcolor: red[500] }} aria-label="recipe">
            R
          </Avatar>
        }
        action={
          <Container>
            <Button>
              <ThumbUpIcon />
            </Button>
            <Button>
              <ThumbDownIcon />
            </Button>
            <Button>
              <ShareIcon />
            </Button>
            <Button>
              <MoreVertIcon />
            </Button>
          </Container>
        }
        title={props.vidInfo.creator}
        subheader="2.1 M subscribers"
      />
    </Card>
  );
}

export default VideoPlayer;
