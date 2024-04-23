import socket
import threading
import sys
import inspect
import pathlib
import json

import http
import cluster
import metadataHandler
import userHandler

import binascii

goBackToReadState = 1

sessionNodeId = threading.local()
userInfo = threading.local()

def doPeriodicCleanup():
    
    deadUsersAndClusters = cluster.clusterManager.listDeadUsers()
    
    for clusterUsersPair in deadUsersAndClusters:
        
        leClusterId = clusterUsersPair[0]
        users = clusterUsersPair[1]
        
        vid = leClusterId[0]
        
        for ip in users:
            
            cluster.clusterManager.removeUserFromACluster( vid, ip )
            pass
        pass
    
    pass


def getIP( addr ):
    return addr[0] + ":" + str( addr[1] )

def sendFavicon( request, sock, addr ):
    
    with open( "favicon.ico", "rb" ) as faviconFile:
        
        faviconData = faviconFile.read()
        return http.sendBinary( sock, faviconData )
    
    return -1

def sendThumbnails( request, sock, addr ):
    
    url = request["url"]
    
    parsedUrl = url.split( "/" )
    
    if len( parsedUrl ) < 3:
        response = http.response404( "Expected Format: website/thumbnails/?videoId=[videoId]" )
        http.sendHTTP( sock, response )
        return 1
        pass
    
    try:
        foo = http.parseURLDict( parsedUrl[2] )
        if not "videoId" in foo:
            response = http.response404( "Expected Format: website/thumbnails/?videoId=[videoId]" )
            http.sendHTTP( sock, response )
            return 1
        
        vid = int( foo["videoId"] )
        
        currentDir = pathlib.Path(__file__).parent.resolve()
        thumbnailDir = currentDir / "videos" / f"video{vid}"
        thumbnailFile = thumbnailDir / "thumbnail.jpg"
        with open( thumbnailFile, "rb" ) as f:
            fileData = f.read()
            http.sendBinary( sock, fileData )
            return 1
            pass
        pass
    except OSError as e:
        response = http.response404( f"Thumbnail {parsedUrl[2]} does not exist." )
        http.sendHTTP( sock, response )
        debugPrint( f"Error = {e}. LINE={inspect.currentframe().f_lineno}" )
        return 1
        pass
    return -1

def catchall( request, sock, addr ):
    print( "handling catchall" )

    def getUUIDFunction( query ):
        #ip = getIP( addr )

        queryDict = http.parseURLDict( query )
        
        if not ( "ip" in queryDict and "port" in queryDict ):
            
            return http.response404( "Expected Format: website/register/?ip=[ipaddr]&port=[port]" )

        ip = queryDict["ip"]

        port = queryDict["port"]

        ipPort = f"{ip}:{port}"

        leUUID = cluster.clusterManager.getUserUUID( ipPort )

        sessionNodeId.leUUID = leUUID

        return http.response200( leUUID )

    try:
        urlList = request["url"].split( "/" )
        if len( urlList ) <3:
            response = http.response404( "Expected Format: website/register/?ip=[ipaddr]&port=[port]" )
            http.sendHTTP( sock, response )
            return goBackToReadState
            pass
        response = getUUIDFunction( urlList[2] )
        pass
    except Exception as e:
        response = http.response404("UUID does not exist for your IP")
        print( f"Exception = {e}, LINE={inspect.currentframe().f_lineno}" )
    print( "genned response" )
    
    http.sendHTTP( sock, response )
    
    return goBackToReadState

def chunk( request, sock, addr ):
    print( "handling chunk" )

    def getChunkFunction( chunk, videoId ):
        print( f"Request addr {addr} wanted video id {videoId}, chunk id {chunk}." )
        videoName = "video" + videoId
        videoDirName = "videos/" + videoName + "/"

        currentDir = pathlib.Path(__file__).parent.resolve()
        videoDir = currentDir / videoDirName
        
        chunkFileName = "None"
        for dirItem in videoDir.iterdir():
            #print( f"checking item = {dirItem.name}" )
            if dirItem.is_file() and dirItem.name.startswith( "playlist" + chunk + "." ):
                chunkFileName = dirItem.name
                break
            pass

        #chunkFileName = "chunk" + chunk + ".webm"
        chunkFile = videoDirName + chunkFileName


        fullChunkPath = str(currentDir) + "/" + chunkFile
        print( f"Current dir = {currentDir}" )
        print( f"Chunk file is {fullChunkPath}", file=sys.stderr )

        try:
            with open( fullChunkPath, "rb" ) as f:
                fileData = f.read()
                http.sendBinary( sock, fileData )
                #response = HttpResponse( fileData, content_type="application/" )
                #response[ "Content-Disposition" ] = 'attachment; filename="' + chunkFileName + '"'
                pass
            pass
        except IOError as e:
            response = http.response404( "DNE" )
            http.sendHTTP( sock, response )
            print( f"error = {e}, LINE={inspect.currentframe().f_lineno}." )
            return goBackToReadState
            #response = HttpResponseNotFound( "DNE" )
            pass

        return goBackToReadState

    urlList = request["url"].split("/")

    if len( urlList ) <3:
        response = http.response404( "Usage: website/chunk/?videoId=[vid]&chunkId=[chunk]" )
        http.sendHTTP( sock, response )
        return goBackToReadState

    queryStr = urlList[2]
    query = http.parseURLDict( queryStr )

    if ( "chunkId" in query ) and ( "videoId" in query ):
        return getChunkFunction( query["chunkId"], query["videoId"] )
    else:
        response = http.response404( "Usage: website/chunk/?videoId=[vid]&chunkId=[chunk]" )
        http.sendHTTP( sock, response )
        return goBackToReadState

    return goBackToReadState

def metadata( request, sock, addr ):
    print( "handling metadata" )

    urlList = request["url"].split("/")

    if len( urlList ) <3:
        response = http.response404( "Usage: website/metadata/?videoId='value'" )
        http.sendHTTP( sock, response )
        return goBackToReadState

    queryStr = urlList[2]
    query = http.parseURLDict( queryStr )
        

    if "videoId" in query:
        vidstr = query["videoId"]
        vid = 0
        try:
            vid = int( vidstr )
            pass
        except ValueError as E:
            response = http.response404( "videoId must be an integer" )
            http.sendHTTP( sock, response )
            print( f"error = {E}, LINE={inspect.currentframe().f_lineno}" )
            return goBackToReadState
        
        stringmd = metadataHandler.leHandler.getMd( vid )

        msg = stringmd
        response = http.response200( msg )
        
        http.sendHTTP( sock, response )
        print( f"Sent: {response}" )
        
        return goBackToReadState

    else:
        
        response = http.response404( "Usage: website/metadata/?videoId='value'" )
        
        http.sendHTTP( sock, response )
        
        return goBackToReadState

    return goBackToReadState

def handleCluster( request, sock, addr ):
    print( "handling cluster" )

    #doPeriodicCleanup()
    errorMessage = "Usage: website.com/cluster/?add=add1,add2,...,addN&rm=rm1,rm2,...,rmM&check=check1,check2,...,checkM&nodeId=id"
    
    urlList = request["url"].split("/")

    if len( urlList ) <3:
        response = http.response404( errorMessage )
        http.sendHTTP( sock, response )
        return goBackToReadState

    queryStr = urlList[2]
    query = http.parseURLDict( queryStr )

    def validateIntQuery( vids ):
        
        for vid in vids:
            try:
                _ = int( vid )
                pass
            except ValueError as e:
                return False
            pass
        return True
    
    
    
    #query = request.GET.dict()

    addVids     = []
    rmVids      = []
    checkVids   = []
    
    if "add" in query:
        addVids = query[ "add" ].split( "," )
        pass

    if "rm" in query:
        rmVids = query[ "rm" ].split( "," )
        pass

    if "check" in query:
        checkVids = query[ "check" ].split( "," )
        pass

    if not ( validateIntQuery( addVids ) and validateIntQuery( rmVids ) and validateIntQuery( checkVids ) and ("nodeId" in query) ):
        response = http.response404( errorMessage )
        http.sendHTTP( sock, response )
        return goBackToReadState

    nodeId = query["nodeId"]

    #requestIP = request.META.get( 'HTTP_X_FORWARDED_FOR' ) or request.META.get( 'HTTP_X_REAL_IP' ) or request.META.get(  )
    if not nodeId in cluster.clusterManager.useridToIp:
        response = http.response404( f"Node id {nodeId} does not exist." )
        http.sendHTTP( sock, response )
        return goBackToReadState

    requestIP = cluster.clusterManager.useridToIp[ nodeId ]

    responseList = []

    for vid in rmVids:
        videoid = int( vid )
        
        cluster.clusterManager.removeUserFromACluster( videoid, requestIP )
        leCluster = cluster.clusterManager.getClusterFromIpVideo( requestIP, videoid )

        if leCluster is not None:
            responseList.append( { "vid": videoid, "operation": "rm", "members": leCluster.members, "timestamps": leCluster.joinTimestamps } )
        else:
            responseList.append( { "vid": videoid, "operation": "rm", "members": "no cluster", "timestamps": "no cluster" } )

        pass

    print( f"newest IP = {requestIP}" )

    for vid in addVids:
        videoid = int( vid )
        cluster.clusterManager.addUserToACluster( videoid, requestIP )

        leCluster = cluster.clusterManager.getClusterFromIpVideo( requestIP, videoid )

        members = leCluster.members
        
        selfIndex = members.index( requestIP )
        refinedCluster = list( members )
        refinedTimestamps = list( leCluster.joinTimestamps )
        
        if selfIndex >= 0:
            del refinedCluster[selfIndex]
            del refinedTimestamps[selfIndex]
            pass

        if requestIP in refinedCluster:
            while True:
                print( "REQUEST IP NOT REMOVED FROM CLUSTER!" )
                pass

        if leCluster is not None:
            responseList.append( { "vid": videoid, "operation": "add", "members": refinedCluster, "timestamps": refinedTimestamps } )
        else:
            responseList.append( { "vid": videoid, "operation": "add", "members": "no cluster", "timestamps": "no cluster" } )

    for videoId in checkVids:
        videoid = int( videoId )
        leCluster = cluster.clusterManager.getClusterFromIpVideo( requestIP, videoid )

        if leCluster is not None:
            responseList.append( { "vid": videoid, "operation": "check", "members": leCluster.members, "timestamps": leCluster.joinTimestamps } )
        else:
            responseList.append( { "vid": videoid, "operation": "check", "members": "no cluster", "timestamps": "no cluster" } )

    #leCluster = cluster.clusterManager.getClusterFromIpVideo( requestIP, videoid )

    responseMsg = json.dumps( responseList )

    print( f"Response = {responseMsg}" )
    
    response = http.response200( responseMsg )
    
    http.sendHTTP( sock, response )

    #return HttpResponse( str( responseList ) )

    return goBackToReadState

def sendManifest( request, sock, addr ):
    
    urlList = request["url"].split("/")
    if( len( urlList ) < 3 ):
        
        response = http.response404( "Expected Format: website/manifest/?videoId=[vid]" )
        http.sendHTTP( sock, response )
        return goBackToReadState
        pass

    query = urlList[2]
    
    queryDict = http.parseURLDict( query )
    
    if not "videoId" in queryDict:
        
        response = http.response404( "Expected Format: website/manifest/?videoId=[vid]" )
        http.sendHTTP( sock, response )
        return goBackToReadState
    
    vid = queryDict["videoId"]
    
    currentDir = pathlib.Path(__file__).parent.resolve()
    manifestDir = currentDir / "videos/"
    
    manifestFile = manifestDir / f"video{vid}/" / "playlist.m3u8"
    
    try:
        with open( manifestFile ) as mf:
            mfContent = mf.read()
            response = http.response200( mfContent )
            http.sendHTTP( sock, response )
            pass
        pass
    except Exception as e:
        response = http.response404( "Does not exist." )
        http.sendHTTP( sock, response )
        print( f"Exception has been reached: {e}, LINE={inspect.currentframe().f_lineno}" )
        pass
    
    return goBackToReadState

def handleLogin( request, sock, addr ):
    
    expectedFormatText = "Expected Format: website/login/?option=[signup|login]&email=[email]&username=[username]"
    # website/login/?opt=[register|login]&email=[leEmail]&username=[leUsername]
    urlList = request["url"].split("/")
    if( len( urlList ) < 3 ):
        
        response = http.response404( expectedFormatText )
        http.sendHTTP( sock, response )
        return goBackToReadState
        pass
    
    query = urlList[2]
    
    queryDict = http.parseURLDict( query )
    
    if not "option" in queryDict:
        response = http.response404( expectedFormatText )
        http.sendHTTP( sock, response )
        return goBackToReadState
    
    option = queryDict["option"]
    
    def subhandlerRegister():
        
        expectedRegisterFormatText = "Expected website/login/?option=signup&email=[email]&username=[username]"
        if ( not "email" in queryDict ) or ( not "username" in queryDict ):
            response = http.response404( expectedRegisterFormatText )
            http.sendHTTP( sock, response )
            return goBackToReadState
            
        email = queryDict["email"]
        username = queryDict["username"]
        
        daGoodStuff = userHandler.leHandler.readFromDB( email )
        
        if not daGoodStuff is None:
            print( ":o" )
            response = http.response404( f"An account already exists with email address {email} (login)." )
            http.sendHTTP( sock, response )
            return goBackToReadState
        # send back salt
        daSalt = userHandler.leHandler.genSalt()

        salthttp = http.response200( daSalt )
        http.sendHTTP( sock, salthttp )

        skeetjr = sock.recv( 4096 )
        skeet = binascii.hexlify( skeetjr ).decode( 'utf-8' )
        #skeet = skeetjr.decode( 'utf-8' )
        securityValue = skeet
        userHandler.leHandler.writeToDB( username, email, daSalt, securityValue )
        userInfo.username = username
        userInfo.email = email
        userInfo.loggedIn = True

        response = http.response200( username )
        http.sendHTTP( sock, response )
        return goBackToReadState
    
    
    def subhandlerLogin():
        expectedLoginFormatText = "Expected website/login/?option=login&email=[email]"
        if not "email" in queryDict:
            response = http.response404( expectedFormatText )
            http.sendHTTP( sock, response )
            return goBackToReadState
            
        email = queryDict["email"]

        # send back salt & challenge
        daGoodStuff = userHandler.leHandler.readFromDB( email )
        
        if daGoodStuff is None:
            response = http.response404( f"User with email address {email} does not exist (create an account)." )
            http.sendHTTP( sock, response )
            return goBackToReadState

        username = daGoodStuff[1]
        salt = daGoodStuff[3]
        pwhash = daGoodStuff[4]
        challenge = userHandler.leHandler.genChallenge()
        
        # Salt is encoded as a string of hex values
        saltChallenge = http.response200( json.dumps( { "salt": salt, "challenge": challenge } ) )
        http.sendHTTP( sock, saltChallenge )
        #sock.sendall( saltChallenge.encode( 'utf-8' ) )
        
        # user sends back hash( hash( salt || password ) || challenge )
        
        skeet = sock.recv( 4096 )
        securityValue = binascii.hexlify( skeet ).decode( 'utf-8' )
        print( f"securityValue = {securityValue}" )
        
        # login if successful, else do not.
        
        deyMatch = userHandler.leHandler.checkUserPassword( challenge, pwhash, securityValue )
        # set flag for being logged in. 
        if deyMatch:
            userInfo.username = username
            userInfo.email = email
            userInfo.loggedIn = True
            pass
        
        # send back "you're logged in" or "nope"
        if deyMatch:
            print( "logging in" )
            response = http.response200( username )
            http.sendHTTP( sock, response )
            pass
        else:
            print( "not logging in" )
            response = http.response404( "Failure" )
            http.sendHTTP( sock, response )
            pass

        return goBackToReadState
    
    
    if option == "login":
        return subhandlerLogin()
    elif option == "signup":
        return subhandlerRegister()
    else:
        response = http.response404( expectedFormatText )
        http.sendHTTP( sock, response )
        return goBackToReadState
    
    return goBackToReadState

def doSecret( request, sock, addr ):
    
    with open("./secret.txt") as f:
        
        response = http.response200( f.read() )
        http.sendHTTP( sock, response )
        pass
    return goBackToReadState

def DNE( request, sock, addr ):
    print( "request endpoint DNE, handling DNE" )

    response = http.response404( "Does not exist" )
    http.sendHTTP( sock, response )

    return goBackToReadState

endpoints = {
    "register": catchall,
    "chunk": chunk,
    "metadata": metadata,
    "cluster": handleCluster,
    "thumbnails": sendThumbnails,
    "manifest": sendManifest,
    "login": handleLogin,
    "favicon.ico": sendFavicon
}


# returns a "next state"
def handleEndpoint( parsedRequest, sock, addr ):
    
    url = parsedRequest["url"]
    
    splitURL = url.split( "/" )
    
    print( f"url = {url}" )
    print( f"splitURL = {splitURL}" )
    
    topDir = splitURL[1]
    
    if topDir in endpoints:
        subhandler = endpoints[topDir]
        pass
    else:
        subhandler = DNE
        pass
    
    try:
        result = subhandler( parsedRequest, sock, addr )
        pass
    except Exception as e:
        print( f"Error = {e}, LINE={inspect.currentframe().f_lineno}" )
        result = goBackToReadState
        pass
    return result


