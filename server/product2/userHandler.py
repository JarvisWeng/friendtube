
import sqlite3
import hashlib
import secrets
import threading

import binascii

databaseName = "users.db"

class userDBDriver:
    
    def initDB( self ):
        self.cur.execute( """
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                salt TEXT NOT NULL,
                password_hash TEXT NOT NULL
            )
        """ )
        self.db.commit()
        
        return
    
    def __init__( self, dbName ):
        
        try:
            self.db = sqlite3.connect( dbName, check_same_thread=False )
            self.cur = self.db.cursor()
            self.initDB()
            pass
        except Exception as E:
            print( f"error while connecting to the db: {E}" )
            pass
        return

    def addEntry( self, username, email, salt, password_hash ):
        
        try:
            self.cur.execute( "BEGIN TRANSACTION" )
            self.cur.execute( """
                INSERT INTO users (username, email, salt, password_hash)
                VALUES (?, ?, ?, ?)
            """, ( username, email, salt, password_hash ) )
            self.db.commit()
            pass
        except Exception as E:
            print( f"Error: {E}" )
            self.db.rollback()
            pass
        return

    def fetchEntry( self, email ):
        try:
            self.cur.execute( "SELECT * FROM users WHERE email = ?;", ( email, ) )
            
            entry = self.cur.fetchone()
            
            return entry
            
        except Exception as E:
            print( f"Error: {E}" )
            pass
        return None

    def removeEntry( self, email ):
        try:
            self.cur.execute( "DELETE FROM users WHERE email = ?;", (email,) )
            self.db.commit()
        except Exception as E:
            print( f"Error: {E}" )
            self.db.rollback()
            pass
        return

    def listAll( self ):
        try:
            self.cur.execute( "SELECT * FROM users;" )
            return self.cur.fetchall()
        except Exception as E:
            print( f"Error: {E}" )
            pass
        
        return None
    pass

dbDriver = userDBDriver( databaseName )

class userHandler:
    
    def __init__( self, dbDriver ):
        self.dbDriver = dbDriver
        # Using one of these so no race conditions occur in db commands.
        self.dbLock = threading.Lock()
        return

    def genSalt( self ):
        saltLenInBits = 256
        saltLenInBytes = saltLenInBits//8
        salt = secrets.token_hex( saltLenInBytes )
        return salt
    
    def genChallenge( self ):
        # TODO: verify that using secrets for both salt and challenge won't introduce a vulnerability.
        challengeLengthInBits = 256
        challengeLengthInBytes = challengeLengthInBits // 8
        challenge = secrets.token_hex( challengeLengthInBytes )
        return challenge
    
    def checkUserPassword( self, challenge, hashedPwCatSalt, whatUserComputes ):
        try:
            # Assume that everything is a string of hex characters
            binHashedPwblah = bytes.fromhex(hashedPwCatSalt)
            challengeCatHashPwCatSalt = binHashedPwblah + challenge.encode( 'utf-8' )
            #print( f"hashedPwCatSalt = {hashedPwCatSalt}" )
            #print( f"challenge = {challenge}" )
            #print( f"challengeCatHashPwCatSalt = {challengeCatHashPwCatSalt}" )
            print( f"whatUserComputes = {whatUserComputes}" )
            CCHPCS = challengeCatHashPwCatSalt #.encode( 'utf-8' ) #bytes.fromhex( challengeCatHashPwCatSalt )
            WUC = whatUserComputes.encode( 'utf-8' ) #bytes.fromhex( whatUserComputes )
            
            hashSlingingSlasher = hashlib.sha256()
            hashSlingingSlasher.update( CCHPCS )
            daHash = hashSlingingSlasher.digest()
            securityValue = binascii.hexlify( daHash ).decode( 'utf-8' )
            print( f"securityValue = {securityValue}" )

            return securityValue == whatUserComputes
        except Exception as E:
            print( f"Error: {E}" )
            return False

    def writeToDB( self, username, email, salt, password_hash ):
        self.dbLock.acquire()
        self.dbDriver.addEntry( username, email, salt, password_hash )
        self.dbLock.release()
        return

    def readFromDB( self, email ):
        self.dbLock.acquire()
        result = self.dbDriver.fetchEntry( email )
        self.dbLock.release()
        return result
    
    pass

leHandler = userHandler( dbDriver )

















