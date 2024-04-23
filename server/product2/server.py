import sys
import socket
import threading
import select
import time

import endpoints
import http

import cluster


maxSimultaneousConnections = 100

port = 3000
host = "127.0.0.1"
#host = "192.168.43.75"
#host = "192.168.43.82"

maxRequestSize = 4096

# 0 - sync
# 1 - async
syncAsyncFlag = 1

eventQueue = []

def isDataReadyToRead( sock ):
    
    
    readList, _, _ = select.select( [sock], [], [], 0 )
    
    return sock in readList

def connectionHandler( sock, addr ):
    print("Got new connection from:", addr)

    state_begin = 0
    state_read  = 1
    state_close = 0xc105e
    state_error = -1
    
    state = state_begin

    alive = True

    while alive:

        #print( f"state = {state}" )
        if state == state_begin:
            state = state_read
            endpoints.userInfo.username = "guest"
            endpoints.userInfo.email = ""
            endpoints.userInfo.loggedIn = False
            endpoints.sessionNodeId.leUUID = None
            pass
        elif state == state_read:
            try:
                recv = sock.recv( maxRequestSize )#.decode( 'utf-8' )
                whatWant = recv.decode( 'utf-8' )
                if not whatWant:
                    print( "What want is null. Check the client terminal." )
                    time.sleep(120)
                    print( "Checking time is over." )
                    pass
                #print( "Thing" )
            except (socket.timeout, socket.error) as E:
                print( f"Socket error: {E}" )
                state = state_close
                continue
            except Exception as E:
                print( f"Some other error occurred: {E}" )
                state = state_close
                continue

            if whatWant:
                parsedReq = http.parseHTTP( whatWant )

                try:
                    state = endpoints.handleEndpoint( parsedReq, sock, addr )
                    pass
                except (socket.timeout, socket.error) as E:
                    print( f"Socket error: {E}" )
                    state = state_close
                    pass
                except Exception as E:
                    print( f"Some other error occurred: {E}" )
                    state = state_close
                    pass
                pass
            else:
                print( f"recv = {recv}" )
                state = state_close
                pass
            pass
        elif state == state_close:
            
            print( f"Connection with address {addr} will now die" )
            sock.close()
            
            try:
                if not endpoints.sessionNodeId.leUUID is None:
                    cluster.clusterManager.yeetUser( endpoints.sessionNodeId.leUUID )
                    pass
                pass
            except Exception as e:
                print( f"error: {e} in main loop" )
                pass
            alive = False
            pass
        elif state == state_error:
            print( f"Achieved an expected error state {state} somehow..." )
            bruh
            pass
        else:
            print( f"Achieved an unexpected error state {state} somehow..." )
            pass
        pass


    if syncAsyncFlag:
        #print( "dying" )
        sys.exit()
    return 0

def doServer( host, port, handler ):
    
    leSocket = socket.socket( socket.AF_INET, socket.SOCK_STREAM )

    leSocket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    leSocket.bind( ( host, port ) )

    leSocket.listen( maxSimultaneousConnections )

    daThreads = []
    
    print("Listening on", host, port)

    while True:
        
        sock, addr = leSocket.accept()

        if syncAsyncFlag == 1:
            leHandler = threading.Thread( target=handler, args=( sock, addr ) )
            leHandler.start()
            daThreads.append( leHandler )
            pass
        else:
            handler( sock, addr )
            pass

        print( "Printing thread states:" )
        for t in daThreads:
            print( f"{t.is_alive()}" )
            pass

        pass
    
    return


def main( args ):
    
    doServer( host, port, connectionHandler )
    
    return 0






main( sys.argv )
