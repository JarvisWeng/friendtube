import socket
import threading

goBackToReadState = 1

statusTexts = {
    200: "OK",
    404: "Not Found"
}

def sendHTTP( sock, httpStr ):
    sock.sendall( httpStr.encode( 'utf-8' ) )
    return

def parseHTTP( req ):
    
    lines = req.split( "\r\n" )

    requestLine = lines[0].split( " " )
    method = requestLine[0]
    url = requestLine[1]
    httpVersion = requestLine[2]

    headers = {}

    for line in lines[1:]:
        if not line:
            break
        key, value = line.split( ": ", 1 )
        headers[key] = value
        pass

    contentIndex = req.find( "\r\n\r\n" )
    content = req[contentIndex + 4:] if contentIndex != -1 else None
    
    result = {
        "method":   method,
        "url":      url,
        "version":  httpVersion,
        "headers":  headers,
        "content":  content
    }
    
    return result

def parseURLDict( leQuery ):
    if leQuery.startswith( "?" ):
        leQuery = leQuery[1:]
        pass

    dictItems = leQuery.split( "&" )
    result = {}
    for assignment in dictItems:
        keyValuePair = assignment.split( "=" )
        if len( keyValuePair) == 2:
            result[keyValuePair[0]] = keyValuePair[1]
            pass
        pass

    return result
    
    

def responseToHTTP( response ):
    
    statusCode = response["statusCode"]
    statusText = response["statusText"]
    content = response["content"]
    contentLen = len( content )
    respAsHTTPStr = f"HTTP/1.1 {statusCode} {statusText}\r\nContent-Length: {len(contentLen)}\r\n\r\n{content}"
    
    return respAsHTTPStr

def response404( message ):
    
    statusCode = 404
    statusText = statusTexts[statusCode]
    content = message
    contentLen = len(content)
    respAsHTTPStr = f"HTTP/1.1 404 Not Found\r\nContent-Length: {contentLen}\r\n\r\n{content}"
    return respAsHTTPStr

def response200( message ):

    content = message
    contentLen = len( content )
    respAsHTTPStr = f"HTTP/1.1 200 OK\r\nContent-Length: {contentLen}\r\n\r\n{content}"
    return respAsHTTPStr

def sendBinary( sock, data ):

    response = f"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {len(data)}\r\n\r\n" 
    
    sock.sendall( response.encode( 'utf-8' ) )
    sock.sendall( data )
    
    return goBackToReadState

