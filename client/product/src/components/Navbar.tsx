import { AppBar,Toolbar, Typography, IconButton, Drawer, List, ListItem, ListItemButton, ListItemIcon, ListItemText, Divider, Box, Avatar, TextField, InputAdornment } from "@mui/material"
import MenuIcon from "@mui/icons-material/Menu"
import VideogameAssetIcon from '@mui/icons-material/VideogameAsset';
import TrendingUpIcon from '@mui/icons-material/TrendingUp';
import LightbulbIcon from '@mui/icons-material/Lightbulb';
import EmojiEventsIcon from '@mui/icons-material/EmojiEvents';
import CheckroomIcon from '@mui/icons-material/Checkroom';
import NewspaperIcon from '@mui/icons-material/Newspaper';
import LiveTvIcon from '@mui/icons-material/LiveTv';
import PodcastsIcon from '@mui/icons-material/Podcasts';
import EmojiEmotionsIcon from '@mui/icons-material/EmojiEmotions';
import LocalMoviesIcon from '@mui/icons-material/LocalMovies';
import AccountCircleIcon from '@mui/icons-material/AccountCircle';
import SubscriptionsIcon from '@mui/icons-material/Subscriptions';
import PlaylistPlayIcon from '@mui/icons-material/PlaylistPlay';
import HistoryIcon from '@mui/icons-material/History';
import LogoutIcon from '@mui/icons-material/Logout';
import SearchIcon from '@mui/icons-material/Search';
import { useState } from "react";
import { red } from "@mui/material/colors";
// function toggleDrawer(drawerOpen : boolean){

// }

function Navbar({LogoClickHandler,LogoutHandler,username}) {
    const [leftDrawer,setLeftDrawer] = useState(false);
    const [rightDrawer,setRightDrawer] = useState(false);

    // console.log(LogoClickHandler);

    function toggleDrawer(open: boolean, drawer:string) {
        return (event: React.KeyboardEvent | React.MouseEvent) => {
          if (
            event.type === 'keydown' &&
            ((event as React.KeyboardEvent).key === 'Tab' ||
              (event as React.KeyboardEvent).key === 'Shift')
          ) {
            return;
          }

          if(drawer == "left"){
            setLeftDrawer(open);
          } else {
            setRightDrawer(open);
          }
        };
    }
    return (
        <AppBar position="fixed">
            <Toolbar variant="dense">
                <IconButton edge="start" color="primary" aria-label="menu" sx={{ mr: 2 }} onClick={toggleDrawer(true,"left")}>
                    <MenuIcon />
                </IconButton>

                <Drawer
                anchor='left'
                open={leftDrawer}
                onClose={toggleDrawer(false,"left")}>
                    <Box
                    sx={{ width:250 }}
                    role="presentation"
                    onClick={toggleDrawer(false,"left")}
                    onKeyDown={toggleDrawer(false,"left")}
                    >
                    <List>
                        {['Trending', 'Gaming', 'Learning', 'Sports', 'Fashion & Beauty', 'News', 'Live', 'Podcasts', 'Comedy', 'Movies & Tv'].map((text, index) => (
                        <ListItem key={text} disablePadding>
                            <ListItemButton>
                            <ListItemIcon>
                                {[<TrendingUpIcon/>, <VideogameAssetIcon/>, <LightbulbIcon/>, <EmojiEventsIcon/>, <CheckroomIcon/>, <NewspaperIcon/>, <LiveTvIcon/>, <PodcastsIcon/>, <EmojiEmotionsIcon/>, <LocalMoviesIcon/>][index]}
                            </ListItemIcon>
                            <ListItemText primary={text} />
                            </ListItemButton>
                        </ListItem>
                        ))}
                    </List>
                    </Box>
                </Drawer>
                <Typography variant="h6" color="primary" component="div" onClick={()=>{LogoClickHandler()}} sx={{ flexGrow: 1 }}>
                    FriendTube
                </Typography>

                <TextField
                margin="dense"
                size="small"
                id="outlined-start-adornment"
                sx={{ m: 1, width: '50ch'}}
                InputProps={{
                    startAdornment: <InputAdornment position="start">
                                        <IconButton type="button" sx={{ p: '10px' }} aria-label="search">
                                            <SearchIcon />
                                        </IconButton>
                                    </InputAdornment>,
                }}
                />
                
                <Typography variant="h6" color="primary" component="div" paddingRight={3} sx={{ flexGrow: 1 }} align="right">
                    {username}
                </Typography>
                <Avatar sx={{ bgcolor: red[500] }} onClick={toggleDrawer(true,"right")}>
                    R
                </Avatar>
                <Drawer
                anchor='right'
                open={rightDrawer}
                onClose={toggleDrawer(false,"right")}>
                    <Box
                    sx={{ width:250 }}
                    role="presentation"
                    onClick={toggleDrawer( false,"right")}
                    onKeyDown={toggleDrawer(false,"right")}
                    >
                    <List>
                        {['Account', 'Subscriptions', 'Playlists', 'History'].map((text, index) => (
                        <ListItem key={text} disablePadding>
                            <ListItemButton>
                            <ListItemIcon>
                                {[<AccountCircleIcon/>, <SubscriptionsIcon/>,<PlaylistPlayIcon/>,<HistoryIcon/>][index]}
                            </ListItemIcon>
                            <ListItemText primary={text} />
                            </ListItemButton>
                        </ListItem>
                        ))}
                    </List>
                    <Divider />
                    <List>
                        {['Logout'].map((text, index) => (
                        <ListItem key={text} disablePadding>
                            <ListItemButton onClick={LogoutHandler}>
                            <ListItemIcon>
                                {[<LogoutIcon/>][index]}
                            </ListItemIcon>
                            <ListItemText primary={text} />
                            </ListItemButton>
                        </ListItem>
                        ))}
                    </List>
                    </Box>
                </Drawer>
            </Toolbar>
        </AppBar>
    )
}

export default Navbar;