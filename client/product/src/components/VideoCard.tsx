import { Avatar, Card, CardActions, CardContent, CardHeader, CardMedia, IconButton, Typography, CardActionArea, Container, Grid } from "@mui/material";
import ShareIcon from '@mui/icons-material/Share';
import MoreVertIcon from '@mui/icons-material/MoreVert';
import { red } from '@mui/material/colors';
import {VideoInfo} from "../Interfaces"

const layout = {
    xs:6,
    sm:5,
    md:4,
    lg:3,
    xl:2
}

function VideoCard({videoClickHandler , info, thumbnail}){
    // console.log(videoClickHandler);
    return (
        <Grid item xs={layout.xs} sm={layout.sm} md={layout.md} lg={layout.lg} xl={layout.xl}>
            <Card sx={{ maxWidth: 345 }} onClick={()=>{videoClickHandler(info)}}>
                <CardActionArea>
                <CardMedia
                    component="img"
                    height="194"
                    image={thumbnail}
                    alt="thumbnail"
                />
                <Container>
                    <Typography variant="body2" color="text.secondary" noWrap={true}>
                    {info.title}
                    </Typography>
                </Container>
                <CardHeader
                    avatar={
                    <Avatar sx={{ bgcolor: red[500] }} aria-label="recipe">
                        R
                    </Avatar>
                    }
                    action={
                    <IconButton aria-label="settings">
                        <MoreVertIcon />
                    </IconButton>
                    }
                    title={info.title}
                    subheader={info.views + " views - " + info.uploadDate}
                />
            </CardActionArea>
            </Card>
        </Grid>
      );
}

export default VideoCard