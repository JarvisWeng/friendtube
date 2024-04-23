import {Box, Button, Grid, Paper, Tab, Tabs, TextField} from "@mui/material"
import { useState } from "react";
import PublishIcon from '@mui/icons-material/Publish';
import LoginIcon from '@mui/icons-material/Login';
import { invoke } from "@tauri-apps/api/tauri";

function LoginPage({onLogin}) {
    const [value, setValue] = useState(0);
    const [error, setError] = useState(false);
    const [username, setUsername] = useState('');
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [confirmation, setConfirmation] = useState('')

    const login = ()=>{
        console.log("loggin in")
        setError(false);

        invoke("get_port").then((port)=>{
            fetch('http://localhost:' + port.toString() + '/login?email=' + email + '&password=' + password, {
              method: 'POST',
              headers: {'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8'} }).then((response)=>{
                console.log(response);
                if (!response.ok) {
                    console.log(response.ok);
                    setError(true);
                } else {
                    response.text().then(
                        (username)=>{
                            console.log(response.ok);
                            onLogin(username)
                        }
                    );
                }
            });
          })
    }

    const signup = ()=>{
        if (password != confirmation){
            setError(true);
            return;
        }

        console.log("sign up")
        setError(false);

        const handler = (response)=>{
            if (!response.ok) {
                console.log("unable to register")
                setError(true);
            } else {
                response.json().then((username)=>
                    onLogin(username)
                )
            }
        }

        invoke("get_port").then((port)=>{
            fetch('http://localhost:' + port.toString() + '/signup?email=' + email + '&username=' + username + '&password=' + password + '&confirmation=' + confirmation, {
              method: 'POST',
              headers: {'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8'} }).then(handler);
          })
    }

    const handleChange = (event: React.SyntheticEvent, newValue: number) => {
        setError(false);
        setValue(newValue);
    };

    return (
        <Grid container spacing={2} sx={{p:20}} color="primary" justifyContent="center">
            <Paper>
                <Box
                component="form"
                sx={{
                '& .MuiTextField-root': { m: 1, width: '25ch' },
                px:10
                }}
                noValidate
                autoComplete="off"
                >
                     <Grid container justifyContent="center" sx={{p:2}}>
                        <Tabs value={value} onChange={handleChange} aria-label="basic tabs example"> 
                            <Tab label="Sign In"  />
                            <Tab label="Sign Up" />
                        </Tabs>
                    </Grid>

                    <div
                    role="tabpanel"
                    hidden={value !== 1}
                    >
                        <div>
                            <TextField
                            error={false}
                            id="outlined-required"
                            label="Username"
                            autoCorrect="off"
                            onChange={e=>{setUsername(e.target.value)}}
                            />
                        </div>
                        <div>
                            <TextField
                            error={false}
                            id="outlined-required"
                            label="email"
                            autoCorrect="off"
                            onChange={e=>{setEmail(e.target.value)}}
                            />
                        </div>
                        <div>
                            <TextField
                            error={error}
                            type="password"
                            id="outlined-required"
                            label="Password"
                            helperText={error ? 'Passwords Do Not Match' : ''}
                            onChange={e=>{setPassword(e.target.value)}}
                            />
                        </div>
                        <div>
                            <TextField
                            error={error}
                            type="password"
                            id="outlined-required"
                            label="Confirm Password"
                            helperText={error ? 'Passwords Do Not Match' : ''}
                            onChange={e=>{setConfirmation(e.target.value)}}
                            />
                        </div>
                        <Grid container justifyContent="center" sx={{p:2}}>
                            <Button variant="contained" startIcon={<PublishIcon />} onClick={signup}>
                                Sign Up
                            </Button>
                        </Grid>
                    </div>

                    <div
                    role="tabpanel"
                    hidden={value !== 0}
                    >
                        <div>
                            <TextField
                            error={error}
                            id="outlined-required"
                            label="Email"
                            onChange={e=>{setEmail(e.target.value)}}
                            />
                        </div>
                        <div>
                            <TextField
                            error={error}
                            id="outlined-required"
                            type="password"
                            label="Password"
                            onChange={e=>{setPassword(e.target.value)}}
                            helperText={error ? 'Invalid Email or Password': ''}
                            />
                        </div>
                        <Grid container justifyContent="center" sx={{p:2}}>
                            <Button variant="contained" startIcon={<LoginIcon />} onClick={login}>
                                Sign In
                            </Button>
                        </Grid>
                    </div>
                </Box>
            </Paper >
        </Grid>
    );
}

export default LoginPage;