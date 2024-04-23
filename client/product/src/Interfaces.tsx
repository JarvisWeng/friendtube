export interface VideoInfo{
    vid:number;
    title:string;
    avatar:any;//img
    creator:string;
    views:number;
    uploadDate:String;
    channel:string;
    numChunks:number;
    duration:number;
    ip:number;
}

export interface VideoChunk {
    vid: number,
    cid: number,
    addr: string
    chunkLengthInBytes: number
    chunkData: Array<number>,
}

export interface VideoState{
    selectState: any;
    info:VideoInfo;
}
  