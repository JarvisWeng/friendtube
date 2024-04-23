
import traceback
import threading
#import readerwriterlock


class ZernettsLock:

    def __init__( self ):
        self.writeMutex = threading.Lock()
        self.noWriters = threading.Lock()

        self.noReaders = threading.Lock()
        self.readMutex = threading.Lock()

        self.numWriters = 0
        self.numReaders = 0

        self.bruhMutex = threading.Lock()

        return

    # Locks are from https://github.com/jzarnett/ece252/blob/master/lectures/compiled/L16-The_Readers-Writers_Problem.pdf

    def acquireWrite( self ):
        if self.bruhMutex.acquire( timeout=60.0 ):
            
            pass
        else:
            print( f"Deadlock detected in acquireWrite with this trace:" )
            print( f"{traceback.format_exc()}" )
            pass
        #self.writeMutex.acquire()
        #self.numWriters += 1
        #if self.numWriters == 1:
        #    self.noReaders.acquire()
        #    pass
        #
        #self.writeMutex.release()
        #self.noWriters.acquire()

        return

    def acquireRead( self ):
        if self.bruhMutex.acquire( timeout=60.0 ):
            
            pass
        else:
            print( f"Deadlock detected in acquireRead with this trace:" )
            print( f"{traceback.format_exc()}" )
            pass
        #self.noReaders.acquire()
        #self.readMutex.acquire()

        #self.numReaders += 1

        #if self.numReaders == 1:
        #    self.noWriters.acquire()
        #    pass

        #self.readMutex.release()
        #self.noReaders.release()
        return

    def releaseRead( self ):
        self.bruhMutex.release()
        #self.readMutex.acquire()
        #self.numReaders -= 1
        #if self.numReaders == 0:
        #    self.noWriters.release()
        #    pass
        #self.readMutex.release()
        
        return

    def releaseWrite( self ):
        self.bruhMutex.release()
        #self.noWriters.release()
        #self.writeMutex.acquire()

        #self.numWriters -= 1

        #if self.numWriters == 0:
        #    self.noReaders.release()
        #    pass

        #self.writeMutex.release()
        
        return
    pass



