
import sys
import pathlib
import json

class metadataHandler:
    
    def __init__( self ):
        videoDirName = "videos/"
        currentDir = pathlib.Path(__file__).parent.resolve()
        videoDir = currentDir / videoDirName
        metadataFile = videoDir / "metadata.json"
        
        mdcontent = '{"bruh":"no worky"}'
        try:
            with open( metadataFile, "r" ) as md:
                
                mdcontent = md.read()
                mdJSON = json.loads( mdcontent )
                self.mdStrs = [ json.dumps( x ) for x in mdJSON ]
                pass
            pass
        except Exception as e:
            print( f"Could not load {metadataFile}" )
            pass
        
        #self.mdJSON = json.loads( mdcontent )
        return
    
    
    def getMd( self, videoId ):
        videoIdm1 = videoId-1
        if videoIdm1 < 0 or videoIdm1 >= len( self.mdStrs ):
            return ":("
        else:
            #return self.mdJSON[ videoIdm1 ]
            return self.mdStrs[ videoIdm1 ]
        pass
    pass


leHandler = metadataHandler()
