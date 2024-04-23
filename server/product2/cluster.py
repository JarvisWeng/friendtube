
import uuid
import time
import threading
import inspect

import readersWritersLock

maxClusterSize = 6

videoIdsWeCurrentlySupport = range(1, 11)

maxClusterTimeoutMinutes = 5000000
maxClusterTimeoutSeconds = 60*maxClusterTimeoutMinutes


debugLogging = True

def debugPrint( leStr ):
    if debugLogging:
        print( leStr )
        pass
    return

class Cluster:
    def __init__( self, videoid ):
        self.videoid        = videoid
        self.maxClusterSize = maxClusterSize
        self.clusterFull    = False
        self.clusterEmpty   = True
        self.members        = []
        self.joinTimestamps = []
        pass

    def splitCluster( self ):
        
        cluster1 = Cluster( self.videoid )
        cluster2 = Cluster( self.videoid )
        cluster1.members = self.members[0::2]
        cluster1.joinTimestamps = self.joinTimestamps[0::2]
        cluster2.members = self.members[1::2]
        cluster2.joinTimestamps = self.joinTimestamps[1::2]
        return ( cluster1, cluster2 )

    # No self is intentional
    def mergeClusters( cluster1, cluster2 ):
        
        leCluster = Cluster( cluster1.videoid )
        leCluster.members           = cluster1.members + cluster2.members
        leCluster.joinTimestamps    = cluster1.joinTimestamps + cluster2.joinTimestamps
        leCluster.clusterEmpty      = len( leCluster.members ) == 0
        leCluster.clusterFull       = len( leCluster.members ) >= leCluster.maxClusterSize
        return leCluster

    def addUser( self, userid ):
        
        if userid in self.members:
            index = self.members.index( userid )
            self.joinTimestamps[index] = time.time() # if already in this, just update the time. 
            return
        # else
        self.members.append( userid )
        self.joinTimestamps.append( time.time() )
        self.clusterEmpty = False
        if len( self.members ) >= self.maxClusterSize:
            self.clusterFull = True
            pass

        return

    def removeUser( self, userid ):
        
        if userid in self.members:
            index = self.members.index( userid )
            self.members.remove(userid)
            del self.joinTimestamps[index]
            self.clusterFull = len( self.members ) >= self.maxClusterSize
            self.clusterEmpty = len( self.members ) > 0
            pass

        return

    def listTimeoutUsers( self ):
        
        now = time.time()
        userips = []
        for userip, timestamp in zip( self.members, self.joinTimestamps ):
            if (now- timestamp) > maxClusterTimeoutSeconds:
                userips.append( userip )
                pass
            pass

        return userips

    pass

class ClusterManager:

    def __init__( self ):
        self.users          = {}    # ip        -> userid
        self.useridToIp     = {}    # userid    -> ip
        self.userClusters   = {}    # userid    -> collection<clusterid>
        self.cluster        = {}    # clusterid -> cluster
        self.cluster    = {}    # videoid   -> collection<clusterid>
        self.changeLog      = []    # Log the changes to clusters so users can update their clusters. 

        self.rwlock_users = readersWritersLock.ZernettsLock()
        self.rwlock_changeLog = readersWritersLock.ZernettsLock()
        self.rwlock_userClusterState = readersWritersLock.ZernettsLock()

        self.clusterCounter = 0
        pass

    def getUserUUID( self, ip : str ) -> str:
        if ip in self.users:
            leUUID = self.users[ip]
            return leUUID

        debugPrint( "Genning new UUID:" )
        leUUID = str( uuid.uuid3( uuid.NAMESPACE_URL, ip ) )
        debugPrint( f"for ip = {ip}, leUUID = {leUUID}" )
        self.rwlock_users.acquireWrite()
        self.useridToIp[leUUID] = ip
        self.users[ip] = leUUID
        self.rwlock_users.releaseWrite()
        # Initialize cluster list
        self.rwlock_userClusterState.acquireWrite()
        self.userClusters[leUUID] = []
        self.rwlock_userClusterState.releaseWrite()

        return leUUID

    def newClusterId( self, videoid ):
        
        clusterNumber = self.clusterCounter
        self.clusterCounter += 1
        
        clusterUUID = str( uuid.uuid3( uuid.NAMESPACE_X500, str( clusterNumber ) ) )
        
        return ( videoid, clusterUUID )

    def initCluster( self, videoids ):
        # This happens before the server starts, so no locks are needed.
        for videoid in videoids:
            firstCluster                = Cluster( videoid )
            clusterId                   = self.newClusterId( videoid )
            #clusterId                   = ( videoid, self.clusterCounter )

            self.cluster[ clusterId ]   = firstCluster
            self.cluster[ videoid ] = [ clusterId ]
            pass
        pass

    def propagateClusterChanges( self, addClusters, changeClusters, removeClusters, alreadyUnderLock=False ):
        
        if not alreadyUnderLock:
            self.rwlock_userClusterState.acquireWrite()

        for clusterIdPair in removeClusters:
            
            leId    = clusterIdPair[0]
            cluster = clusterIdPair[1]
            videoid = cluster.videoid
            # Remove from each user clusters:

            for user in cluster.members:
                
                #userid = self.getUserUUID( str( user ) )

                if not user in self.users:
                    debugPrint( f"Probably desynch? user id {user} exists in cluster with id {leId}, but no userid has been registered with that. LINE={inspect.currentframe().f_lineno}" )
                    pass
                userid = self.users[user]

                self.userClusters[userid].remove( leId )

                self.changeLog.append( ( userid, "-", videoid, None ) )
                pass
            
            # Remove from the cluster
            self.cluster[videoid].remove( leId )
            del self.cluster[leId]

            pass

        # This part just says "inform these users of the change of cluster definition."
        for clusterIdPair in changeClusters:
            leId    = clusterIdPair[0]
            cluster = clusterIdPair[1]
            videoid = cluster.videoid

            for user in cluster.members:
                self.changeLog.append( ( user, ".", videoid, leId ) )
                pass

            pass

        for clusterIdPair in addClusters:
            
            leId    = clusterIdPair[0]
            cluster = clusterIdPair[1]
            videoid = cluster.videoid

            debugPrint( f"adding cluster id = {leId}, videoid = {videoid}, members = {cluster.members if cluster is not None else 'None'} LINE={inspect.currentframe().f_lineno}" )
            
            for user in cluster.members:
                #userid = self.getUserUUID( str( user ) )
                if not user in self.users:
                    debugPrint( f"Desynch was observed, user {user} was in cluster with id {leId} but not in self.users LINE={inspect.currentframe().f_lineno}." )
                userid = self.users[user]
                if not leId in self.userClusters[userid]:
                    self.userClusters[userid].append( leId )

                self.changeLog.append( ( user, "+", videoid, leId ) )
                pass

            self.cluster[videoid].append( leId )
            self.cluster[leId] = cluster
            pass

        if not alreadyUnderLock:
            self.rwlock_userClusterState.releaseWrite()
        return

    def addUserToACluster( self, videoid, ip ):

        userId          = self.getUserUUID( ip )

        self.rwlock_userClusterState.acquireWrite()

        leCluster   = list( self.cluster[ videoid ] )
        
        debugPrint( f"userId = {userId} for ip = {ip}" )
        
        for clusterid in leCluster:
            
            if not clusterid in self.cluster:
                debugPrint( "Observed desynch when adding a user to a cluster." )
                debugPrint( f"clusterid = {clusterid} in leClusterFuck, but not in self.cluster LINE={inspect.currentframe().f_lineno}." )
            elif self.cluster[clusterid] is None:
                debugPrint( "Bug spotted in add user to cluster." )
                debugPrint( f"self.cluster[{clusterid}] is None. It is never expected that None will be a thing here LINE={inspect.currentframe().f_lineno}. " )
            elif not self.cluster[clusterid].clusterFull or ip in self.cluster[clusterid].members:

                self.cluster[clusterid].addUser( ip )
                if not clusterid in self.userClusters[userId]:
                    self.userClusters[ userId ].append( clusterid )
                leCluster = self.cluster[clusterid]

                self.propagateClusterChanges( [], [ ( clusterid, leCluster ) ], [], alreadyUnderLock=True )
                self.rwlock_userClusterState.releaseWrite()
                return

            pass

        splitThisClusterid = None

        debugPrint( "No suitable cluster found, so we are splitting a candidate:" )
        # None found, split one:
        for clusterid in leCluster:
            
            if not clusterid in self.cluster:
                debugPrint( "Observed desynch when adding a user to a cluster." )
                debugPrint( f"clusterid = {clusterid} in leClusterFuck, but not in self.cluster LINE={inspect.currentframe().f_lineno}." )
            elif self.cluster[clusterid] is None:
                debugPrint( "Bug spotted in add user to cluster." )
                debugPrint( f"self.cluster[{clusterid}] is None. It is never expected that None will be a thing here LINE={inspect.currentframe().f_lineno}. " )
            elif self.cluster[clusterid].clusterFull:
                splitThisClusterid = clusterid
                break
            pass

        #print( "user cluster:" )
        #for userid, clusterid in self.userClusters.items():
        #    print( f"userid = {userid}, clusterid = {clusterid}" )
        #print( "ip maps:" )
        #for ip_, userid_ in self.users.items():
        #    print( f"ip = {ip_}, userid = {userid_}" )

        if splitThisClusterid is not None:

            if not splitThisClusterid in self.cluster:
                debugPrint( "Desynch has been observed when adding a user to a cluster." )
                debugPrint( f"When attempting to split the cluster, id {splitThisClusterid} no longer exists in self.cluster LINE={inspect.currentframe().f_lineno}" )
                self.rwlock_userClusterState.releaseWrite()
                return
                
            cluster_ = self.cluster[splitThisClusterid]
            if cluster_ is None:
                debugPrint( "Bug spotted in add user to cluster." )
                debugPrint( f"When attempting to split the cluster, self.cluster[{splitThisClusterid}] is None. It is never expected that None will be a thing here. LINE={inspect.currentframe().f_lineno}" )
                self.rwlock_userClusterState.releaseWrite()
                return
            cluster_.addUser( ip )

            if not userId in self.userClusters:
                debugPrint( "Desynch has been observed when adding a user to a cluster." )
                debugPrint( f"When attempting to split the cluster, id {userId} no longer exists in self.cluster LINE={inspect.currentframe().f_lineno}" )
                self.rwlock_userClusterState.releaseWrite()
                return
            elif self.userClusters[userId] is None:
                debugPrint( "Bug spotted in add user to cluster." )
                debugPrint( f"When attempting to split the cluster, self.cluster[{splitThisClusterid}] is None. It is never expected that None will be a thing here. LINE={inspect.currentframe().f_lineno}" )
                self.rwlock_userClusterState.releaseWrite()
                return

            if not splitThisClusterid in self.userClusters[ userId ]:
                self.userClusters[ userId ].append( splitThisClusterid )

            clusterIdPair = ( splitThisClusterid, cluster_ )
            mitosis = cluster_.splitCluster()
            
            newClusterId1 = self.newClusterId( videoid )
            newClusterId2 = self.newClusterId( videoid )
            
            cluster1IdPair = ( newClusterId1, mitosis[0] )
            cluster2IdPair = ( newClusterId2, mitosis[1] )
            
            addList = [ cluster1IdPair, cluster2IdPair ]
            removeList = [ clusterIdPair ]

            self.propagateClusterChanges( addList, [], removeList, alreadyUnderLock=True )
            pass
        else:
            debugPrint( f"Desynch has been observed in add user to a cluster. All clusters were full in the first part, but none were full in the second part. LINE={inspect.currentframe().f_lineno}" )
            pass
        self.rwlock_userClusterState.releaseWrite()
        return

    def removeUserFromACluster( self, videoid, ip ):
        
        
        leClusterid = None
        tryMerge = False

        userId = self.getUserUUID( str( ip ) )

        self.rwlock_userClusterState.acquireWrite()
        
        for clusterid in self.userClusters[userId]:
            if clusterid[0] == videoid:
                self.cluster[clusterid].removeUser( ip )
                
                tryMerge = len( self.cluster[clusterid].members) < ( self.cluster[clusterid].maxClusterSize / 2 )
                leClusterid = clusterid

                while clusterid in self.userClusters[userId]:
                    self.userClusters[userId].remove( clusterid )
                
                break
            pass

        

        MergeSuccessful = False
        if tryMerge:
            for candid in self.cluster[videoid]:
                if ( candid != leClusterid ) and ( len( self.cluster[candid].members ) < self.cluster[candid].maxClusterSize/2 ):
                    
                    # Merge
                    
                    #self.rwlock_userClusterState.acquireRead()
                    clusterSrc1 = self.cluster[leClusterid]
                    clusterSrc2 = self.cluster[candid]
                    #self.rwlock_userClusterState.releaseRead()
                    
                    mergedCluster = Cluster.mergeClusters( clusterSrc1, clusterSrc2 )
                    mergedClusterId = self.newClusterId( videoid )
                    
                    addList = [ ( mergedClusterId, mergedCluster ) ]
                    removeList = [ ( leClusterid, clusterSrc1 ), ( candid, clusterSrc2 ) ]

                    self.propagateClusterChanges( addList, [ ], removeList, True )

                    self.rwlock_userClusterState.releaseWrite()
                    return
                pass
            pass
        # If no merging is required OR it failed to find a good candidate to merge:
        if leClusterid is not None:
            self.propagateClusterChanges( [], [ ( leClusterid, self.cluster[leClusterid] ) ], [], True )
            pass

        self.rwlock_userClusterState.releaseWrite()
        return

    def yeetUser( self, nodeId ):
        
        self.rwlock_userClusterState.acquireRead()
        if not nodeId in self.useridToIp:
            debugPrint( f"node id {nodeId} does not exist in the self.useridToIp dict LINE={inspect.currentframe().f_lineno}" )
            debugPrint( f"self.useridToIp: {self.useridToIp}" )
            debugPrint( f"self.users: {self.users}" )
            self.rwlock_userClusterState.releaseRead()
            return
        ip = self.useridToIp[ nodeId ]
        if not nodeId in self.userClusters:
            debugPrint( f"node id {nodeId} does not exist in the self.userClusters dict LINE={inspect.currentframe().f_lineno}" )
            self.rwlock_userClusterState.releaseRead()
            return
        videoIds = list( set( [ x[0] for x in self.userClusters[nodeId] ] ) )
        self.rwlock_userClusterState.releaseRead()
        
        for vid in videoIds:
            self.removeUserFromACluster( vid, ip )
            
            pass
        
        return


    def getClusterFromIpVideo( self, ip, videoid ):

        userid = self.getUserUUID( ip )

        self.rwlock_userClusterState.acquireRead()
        #print( f"getting ip {ip}, user {userid}, video {videoid}" )
        #print( f"user's clusters = {self.userClusters[userid]}" )
        #print( f"clusters = {self.cluster}" )
        for clusterid in self.userClusters[userid]:
            if clusterid not in self.cluster:
                debugPrint( "Observed desynch while getting cluster from ip, videoid." )
                debugPrint( f"clusterid = {clusterid} in leClusterFuck, but not in self.cluster LINE={inspect.currentframe().f_lineno}." )
                debugPrint( f"clusters: {self.userClusters[userid]}" )
                debugPrint( f"ip = {ip}, userid = {userid}, numclusters = {len(self.cluster)}. " )
                for key,value in self.cluster.items():
                    debugPrint( f"self.cluster contains {key}" )
                    pass
                self.rwlock_userClusterState.releaseRead()
                return None
            elif self.cluster[clusterid] is None:
                debugPrint( "Observed bug while getting cluster from ip, videoid." )
                debugPrint( f"self.cluster[{clusterid}] is None, but we never expect that." )
                self.rwlock_userClusterState.releaseRead()
                return None
            else:
                leCluster = self.cluster[ clusterid ]
            
                if leCluster.videoid == videoid:
                    self.rwlock_userClusterState.releaseRead()
                    return leCluster
                pass
            pass
        self.rwlock_userClusterState.releaseRead()
        debugPrint( f"Not found cluster from ip {ip} (userid {userid}), videoid {videoid} " )
        return None

    def listDeadUsers( self ):
        
        deadUsersAndClusters = []
        self.rwlock_userClusterState.acquireRead()
        for videoid, clusterList in self.cluster.items():
            for clusterid in clusterList:
                clutter = self.cluster[clusterid]
                deadUsers = clutter.listTimeoutUsers()
                deadUsersAndClusters.append( ( clusterid, deadUsers ) )
                pass
            pass
        self.rwlock_userClusterState.releaseRead()
        return deadUsersAndClusters
    pass

clusterManager = ClusterManager()
clusterManager.initCluster( videoIdsWeCurrentlySupport )
