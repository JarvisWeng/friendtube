use bytecodec::bytes::BytesEncoder;
use bytecodec::bytes::RemainingBytesDecoder;
use bytecodec::io::IoDecodeExt;
use bytecodec::io::IoEncodeExt;
use bytecodec::Encode;
use httpcodec::{
    BodyDecoder,
    BodyEncoder,
    HttpVersion,
    Method,
    ReasonPhrase,
    Request,
    RequestDecoder,
    RequestEncoder,
    RequestTarget,
    Response,
    ResponseDecoder,
    ResponseEncoder,
    StatusCode,
    HeaderField,
};
use std::str::FromStr;

//encoder-decoder for http requests
pub struct HttpCodec;

impl HttpCodec {

    fn convert_from_http_req(mut request: http::Request<Vec<u8>>)->Request<Vec<u8>>{
        let method = Method::new(request.method().as_str()).unwrap();//fine with simple panic here
        let target = match request.uri().path_and_query() {
            Some(p)=>{
                let t = p.as_str();
                match RequestTarget::new(t) {
                    Ok(s)=>s,
                    Err(e)=>{//there should be no request with invisible ascii characters
                        println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_req in creating target for uri = {}",request.uri());
                        println!("[HTTP CODEC] ERROR = {}",e.to_string());
                        panic!();
                    }
                }
            },
            None=>{
                println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_req no path found in uri = {}",request.uri());
                panic!();//panic for now and see whats the problem. TODO: remove in demo
            }
        };

        let mut req = Request::new(
            method,
            target,
            HttpVersion::V1_1,
            request.body().to_owned()
        );

        for (k,v) in request.headers_mut(){
            let v = match v.to_str(){
                Ok(s)=>s,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_req while converting header value to str");
                    println!("[HTTP CODEC] val = {:?}",v);
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            let h = match HeaderField::new(k.as_str(),v){
                Ok(s)=>s,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_req while adding header");
                    println!("[HTTP CODEC] header name, value = {}, {}",k.as_str(),v);
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            req.header_mut().add_field(h);  
        }

        req
    }

    fn convert_to_http_req(request: Request<Vec<u8>>)->http::Request<Vec<u8>>{

        let (head, body) = request.take_body();
        let mut ret = match http::Request
            ::builder()
            .method(head.method().as_str())
            .uri(head.request_target().as_str())
            .body(body) 
            {
                Ok(r)=>r,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_to_http_req");
                    println!("[HTTP CODEC] req head = {:?}",head);
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };

        let headers = ret.headers_mut();
        for f in head.header().fields() {
            let name: http::HeaderName = match http::HeaderName::from_str(f.name()){
                Ok(r)=>r,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_to_http_req while adding header name");
                    println!("[HTTP CODEC] header name = {:?}",f.name());
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            let val = match http::HeaderValue::from_str(f.value()){
                Ok(r)=>r,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_to_http_req while adding header val");
                    println!("[HTTP CODEC] val = {}",f.value());
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            headers.insert(name, val);
        }
        ret
    }

    fn convert_from_http_resp(mut response: http::Response<Vec<u8>>)->Response<Vec<u8>>{
        let scode = match StatusCode::new(response.status().as_u16()){
            Ok(sc)=>sc,
            Err(e)=>{
                println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_resp while adding status code");
                println!("[HTTP CODEC] status code = {}",response.status().as_u16());
                println!("[HTTP CODEC] ERROR = {}",e.to_string());
                panic!();
            }
        };
        let rp = match response.status().canonical_reason(){
            Some(sc)=>{
                match ReasonPhrase::new(sc){
                    Ok(r)=>r,
                    Err(e)=>{
                        println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_resp while adding reason phrase");
                        println!("[HTTP CODEC] reason phrase = {}",sc);
                        println!("[HTTP CODEC] ERROR = {}",e.to_string());
                        panic!();
                    }
                }
            }
            None=>{
                ReasonPhrase::new("").unwrap()
            }
        };
        let mut resp = Response::new(
            HttpVersion::V1_1,
            scode,
            rp,
            response.body().to_owned()
        );

        for (k,v) in response.headers_mut(){
            let v = match v.to_str(){
                Ok(s)=>s,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_resp while converting header value to str");
                    println!("[HTTP CODEC] val = {:?}",v);
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            let h = match HeaderField::new(k.as_str(),v){
                Ok(s)=>s,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_from_http_resp while adding header");
                    println!("[HTTP CODEC] header name, value = {}, {}",k.as_str(),v);
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            resp.header_mut().add_field(h);   
        }

        resp 
    }

    fn convert_to_http_resp(response: Response<Vec<u8>>)->http::Response<Vec<u8>>{
        let mut ret = match http::Response
            ::builder()
            .status(response.status_code().as_u16())
            .version(http::version::Version::HTTP_11)
            .body(response.body().to_owned())
            {
                Ok(r)=>r,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_to_http_resp");
                    println!("[HTTP CODEC] req = {:?}",response);
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };

        let headers = ret.headers_mut();
        for f in response.header().fields() {
            let name: http::HeaderName = match http::HeaderName::from_str(f.name()){
                Ok(r)=>r,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_to_http_resp while adding header name");
                    println!("[HTTP CODEC] header name = {:?}",f.name());
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            let val = match http::HeaderValue::from_str(f.value()){
                Ok(r)=>r,
                Err(e)=>{
                    println!("[HTTP CODEC] ERROR in HttpCodec::convert_to_http_resp while adding header val");
                    println!("[HTTP CODEC] val = {}",f.value());
                    println!("[HTTP CODEC] ERROR = {}",e.to_string());
                    panic!();//panic for now and see whats the problem. TODO: remove in demo
                }
            };
            headers.insert(name, val);
        }
        ret
    }

    pub fn encode_request(mut request: http::Request<Vec<u8>>) -> Vec<u8> {
        let req = HttpCodec::convert_from_http_req(request);
        let mut req_encoder = RequestEncoder::new(BodyEncoder::new(BytesEncoder::new()));
        req_encoder.start_encoding(req).expect("Error while starting to encode");
        let mut buf: Vec<u8> = Vec::new();
        req_encoder.encode_all(&mut buf).expect("Could not finish encoding");
        return buf;
    }

    pub fn encode_response(mut response: http::Response<Vec<u8>>) -> Vec<u8> {
        let mut rsp = HttpCodec::convert_from_http_resp(response);

        let mut rsp_encoder = ResponseEncoder::new(BodyEncoder::new(BytesEncoder::new()));
        rsp_encoder.start_encoding(rsp).expect("Error while starting to encode");
        let mut buf: Vec<u8> = Vec::new();
        rsp_encoder.encode_all(&mut buf).expect("Could not finish encoding");
        return buf;
    }

    pub fn decode_response(response: Vec<u8>) -> http::Response<Vec<u8>> {
        let buf: &[u8] = response.as_ref();
        // println!("{:?}", buf);
        
        let mut rsp_decoder = ResponseDecoder::<BodyDecoder<RemainingBytesDecoder>>::default();

        let rsp = rsp_decoder.decode_exact(buf).expect("Error while decoding");

        HttpCodec::convert_to_http_resp(rsp)
    }

    pub fn decode_request(request: Vec<u8>) -> http::Request<Vec<u8>> {
        let buf: &[u8] = request.as_ref();
        
        let mut req_decoder = RequestDecoder::<BodyDecoder<RemainingBytesDecoder>>::default();
        
        let req = req_decoder.decode_exact(buf).expect("Error while decoding");
        
        HttpCodec::convert_to_http_req(req)
    }
}
