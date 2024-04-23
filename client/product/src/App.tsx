import { useState } from "react";
import "./App.css";
import Navbar from "./components/Navbar";
import VideoGridPage from "./pages/VideoGridPage";
import VideoPlayPage from "./pages/VideoPlayPage";
import LoginPage from "./pages/LoginPage";
import { VideoInfo } from "./Interfaces";
import { createTheme, ThemeProvider } from "@mui/material";
import { amber, yellow, grey } from "@mui/material/colors";
import CssBaseline from "@mui/material/CssBaseline";
const theme = createTheme({
  palette: {
    mode: "dark",
    primary: amber,
    secondary: yellow,
  },
});

enum Page {
  Login = 0,
  Home,
  Video,
}

function App() {
  const [pageSelect, setPageSelect] = useState<Page>(Page.Login);
  const [vidInfo, setVidInfo] = useState<VideoInfo>();
  const [username, setUsername] = useState<String>('');

  const videoClickHandler = (info: VideoInfo) => {
    setVidInfo(info);
    setPageSelect(Page.Video);
  };

  const LogoClickHandler = () => {
    setPageSelect(Page.Home);
  };
  // console.log(ip);

  var page;
  switch (pageSelect) {
    case Page.Login:
      page = (
        <LoginPage
          onLogin={(username) => {
            setUsername(username);
            setPageSelect(Page.Home);
          }}
        ></LoginPage>
      );
      break;
    case Page.Home:
      page = (
        <>
          <Navbar
            LogoClickHandler={LogoClickHandler}
            LogoutHandler={() => {
              setPageSelect(Page.Login);
            }}
            username={username}
          ></Navbar>
          <VideoGridPage videoClickHandler={videoClickHandler}></VideoGridPage>
        </>
      );
      break;
    case Page.Video:
      page = (
        <>
          <Navbar
            LogoClickHandler={LogoClickHandler}
            LogoutHandler={() => {
              setPageSelect(Page.Login);
            }}
            username={username}
          ></Navbar>
          <VideoPlayPage vidInfo={vidInfo}></VideoPlayPage>
        </>
      );
      break;
  }

  // console.log(theme);
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline enableColorScheme />
      {page}
    </ThemeProvider>
  );
}

export default App;
