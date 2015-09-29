// extern crate regex;

use std::net::TcpStream;
use std::io::prelude::*;
use std::str;
use std::collections::HashMap;

use std::error::Error;
use std::path::Path;
use std::fs::File;
#[cfg(test)]
mod tests{

    fn test_content_type(req: &str, proof: &str) {
        assert_eq!(super::Response::get_content_type(req), proof);
    }

    fn test_method(req: &str, proof: &str) {
        let method = super::parse_method(req);
        assert_eq!(method, proof);
    }

    fn test_path(req: &str, proof: &str) {
        let path = super::parse_path(req);
        assert_eq!(path, proof);
    }

    fn test_version(req: &str, proof: &str) {
        let version = super::parse_version(req);
        assert_eq!(version, proof);
    }

    #[test]
    fn test_parse_header(){
        let header_string = "GET / HTTP/1.1
        Host: localhost:8080
        User-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:41.0) Gecko/20100101 Firefox/41.0
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
        Accept-Language: en-US,en;q=0.5
        Accept-Encoding: gzip, deflate
        DNT: 1
        Cookie: _ga=GA1.1.395469911.1442866274
        Connection: keep-alive
        ".to_string();
                let headers = super::parse_headers(header_string);

                assert_eq!(headers["DNT"], "1");
                assert_eq!(headers["Host"], "localhost:8080")
    }

    #[test]
    fn test_parse_method() {
        test_method("GET / HTTP/1.1", "GET");

        test_method("GET /test HTTP/1.1", "GET");

        test_method("PUT /test HTTP/1.1", "PUT");

        test_method("POST / HTTP/1.1", "POST");

        test_method("GET / HTTP/1.0", "GET");
    }

    #[test]
    fn test_parse_path() {
        test_path("GET / HTTP/1.1", "/");

        test_path("GET /test HTTP/1.1", "/test");

        test_path("PUT /test HTTP/1.1", "/test");

        test_path("POST / HTTP/1.1", "/");

        test_path("GET / HTTP/1.0", "/");

    }

    #[test]
    fn test_parse_version() {
        test_version("GET /test HTTP/1.1", "HTTP/1.1");

        test_version("PUT / HTTP/1.0", "HTTP/1.0");

        test_version("POST / HTTP/1.1", "HTTP/1.1");

        test_version("GET / HTTP/1.0" ,"HTTP/1.0");
    }

    #[test]
    fn test_parse_content_type() {
        test_content_type("index.html", "text/html");
        test_content_type("style.css", "text/css");
        test_content_type("script.js", "text/javascript");
        test_content_type("some.bs_extension", "text/plain");
    }
}
#[derive(Debug)]
pub struct Request {
    pub stream: TcpStream,
    pub response: Response,
    pub path: String,
    pub method: String,
    pub host: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Response {
    pub body: String,
}

pub struct Header {
    pub name: String,
    pub value: String,
}

impl Request {
    pub fn new(mut stream: TcpStream) -> Request {
        let mut buf = [0; 1024];
        stream.read(&mut buf).unwrap();
        let s = str::from_utf8(&buf).unwrap();
        let headers = parse_headers(s.to_string());
        Request {
            stream: stream,
            response: Response::new(),
            path: String::new(),
            method: String::new(),
            host: String::new(),
            headers: headers,
        }
    }

}

impl Response {
    pub fn new() -> Response {
        Response {
            body: String::new()
        }
    }
    pub fn to_response(&mut self, headers: HashMap<String, String>, serve_directory: String) -> &[u8] {
        // let self.body = self.make_body(headers);
        self.make_body(headers, serve_directory);
        self.body.as_bytes()
    }
    fn make_body(&mut self, headers: HashMap<String, String>, serve_directory: String) {
        let method = &headers["Method"];
        let default_index = "index.html".to_string();
        let path = match headers["Path"].as_ref() {
            "/" => &default_index,
            _ => &headers["Path"],
        };
        // if path == "/" {
        //     let path = "/index.html";
        // }
        // println!("About to look for {}{}", serve_directory, path);
        let file_path = format!("{}/{}", serve_directory, path);
        let version = headers["Version"].clone();
        let response_text = match self.get_text(&file_path) {
            Some(text) => text,
            None => "Content Not Found".to_string(),
        };
        let status = match self.get_text(&file_path) {
            Some(_) => "200 OK".to_string(),
            None => "404 Not Found".to_string(),
        };
        let content_type = Response::get_content_type(&path);
        println!("{} - {} - {} - {:?}", status, method, path, headers);
        let response_header = format!("{} {}\r\nContent-Type: {}\r\n\r\n", version, status, content_type);
        let response_body = format!("{}", response_text);
        self.body = format!("{}{}\r\n", response_header, response_body);
    }

    fn get_text(&self, path: &str) -> Option<String> {
        let path = Path::new(path);
        let display = path.display();

        let mut file = match File::open(&path) {
            // The `description` method of `io::Error` returns a string that
            // describes the error
            Err(why) => {
                println!("couldn't open {}: {} ({})", display,
                                                 Error::description(&why), why);
                return None;
            },
            Ok(file) => file,
        };
        let metadata = match file.metadata() {
            Ok(md) => md,
            Err(why) => {
                println!("Couldn't get metadata about {}: {} ({})", display,
                                                Error::description(&why), why);
                return None;
            }
        };

        if metadata.is_dir() {
            println!("{} is a directory!", display);
            return None;
        }
        let mut s = String::new();
        let content = match file.read_to_string(&mut s) {
            Err(why) => {
                println!("couldn't read {}: {} ({})", display,
                                                 Error::description(&why), why);
                return None;
            }
            Ok(_) => Some(s),
        };

        content
    }

    pub fn get_content_type(filename: &str) -> &str {
        // let formats =
        let format = match filename.split(".").last() {
            Some(format) => format,
            None => "txt"
        };
        // println!("format is {}", format);
        let content_type = Response::content_type_for(format);

        // println!("{} is {}", format, content_type);
        content_type
    }

    pub fn content_type_for(format: &str) -> &str {
        let mut types = HashMap::new();
        types.insert("html", "text/html");
        types.insert("css", "text/css");
        types.insert("js", "text/javascript");
        types.insert("p12", "application/x-pkcs12");
        types.insert("p7m", "application/pkcs7-mime");
        types.insert("p7s", "application/pkcs7-signature");
        types.insert("p7r", "application/x-pkcs7-certreqresp");
        types.insert("p7b", "application/x-pkcs7-certificates");
        types.insert("p8", "application/pkcs8");
        types.insert("plf", "application/vnd.pocketlearn");
        types.insert("pnm", "image/x-portable-anymap");
        types.insert("pbm", "image/x-portable-bitmap");
        types.insert("pcf", "application/x-font-pcf");
        types.insert("pfr", "application/font-tdpfr");
        types.insert("pgn", "application/x-chess-pgn");
        types.insert("pgm", "image/x-portable-graymap");
        types.insert("png", "image/png");
        types.insert("ppm", "image/x-portable-pixmap");
        types.insert("pskcxml", "application/pskc+xml");
        types.insert("pml", "application/vnd.ctc-posml");
        types.insert("ai", "application/postscript");
        types.insert("pfa", "application/x-font-type1");
        types.insert("pbd", "application/vnd.powerbuilder6");
        // types.insert("", "application/pgp-encrypted");
        types.insert("pgp", "application/pgp-signature");
        types.insert("box", "application/vnd.previewsystems.box");
        types.insert("ptid", "application/vnd.pvi.ptid1");
        types.insert("pls", "application/pls+xml");
        types.insert("str", "application/vnd.pg.format");
        types.insert("ei6", "application/vnd.pg.osasli");
        types.insert("dsc", "text/prs.lines.tag");
        types.insert("psf", "application/x-font-linux-psf");
        types.insert("qps", "application/vnd.publishare-delta-tree");
        types.insert("wg", "application/vnd.pmi.widget");
        types.insert("qxd", "application/vnd.quark.quarkxpress");
        types.insert("esf", "application/vnd.epson.esf");
        types.insert("msf", "application/vnd.epson.msf");
        types.insert("ssf", "application/vnd.epson.ssf");
        types.insert("qam", "application/vnd.epson.quickanime");
        types.insert("qfx", "application/vnd.intu.qfx");
        types.insert("qt", "video/quicktime");
        types.insert("rar", "application/x-rar-compressed");
        types.insert("ram", "audio/x-pn-realaudio");
        types.insert("rmp", "audio/x-pn-realaudio-plugin");
        types.insert("rsd", "application/rsd+xml");
        types.insert("rm", "application/vnd.rn-realmedia");
        types.insert("bed", "application/vnd.realvnc.bed");
        types.insert("mxl", "application/vnd.recordare.musicxml");
        types.insert("musicxml", "application/vnd.recordare.musicxml+xml");
        types.insert("rnc", "application/relax-ng-compact-syntax");
        types.insert("rdz", "application/vnd.data-vision.rdz");
        types.insert("rdf", "application/rdf+xml");
        types.insert("rp9", "application/vnd.cloanto.rp9");
        types.insert("jisp", "application/vnd.jisp");
        types.insert("rtf", "application/rtf");
        types.insert("rtx", "text/richtext");
        types.insert("link66", "application/vnd.route66.link66+xml");
        types.insert("rss, .xml", "application/rss+xml");
        types.insert("shf", "application/shf+xml");
        types.insert("st", "application/vnd.sailingtracker.track");
        types.insert("svg", "image/svg+xml");
        types.insert("sus", "application/vnd.sus-calendar");
        types.insert("sru", "application/sru+xml");
        types.insert("setpay", "application/set-payment-initiation");
        types.insert("setreg", "application/set-registration-initiation");
        types.insert("sema", "application/vnd.sema");
        types.insert("semd", "application/vnd.semd");
        types.insert("semf", "application/vnd.semf");
        types.insert("see", "application/vnd.seemail");
        types.insert("snf", "application/x-font-snf");
        types.insert("spq", "application/scvp-vp-request");
        types.insert("spp", "application/scvp-vp-response");
        types.insert("scq", "application/scvp-cv-request");
        types.insert("scs", "application/scvp-cv-response");
        types.insert("sdp", "application/sdp");
        types.insert("etx", "text/x-setext");
        types.insert("movie", "video/x-sgi-movie");
        types.insert("ifm", "application/vnd.shana.informed.formdata");
        types.insert("itp", "application/vnd.shana.informed.formtemplate");
        types.insert("iif", "application/vnd.shana.informed.interchange");
        types.insert("ipk", "application/vnd.shana.informed.package");
        types.insert("tfi", "application/thraud+xml");
        types.insert("shar", "application/x-shar");
        types.insert("rgb", "image/x-rgb");
        types.insert("slt", "application/vnd.epson.salt");
        types.insert("aso", "application/vnd.accpac.simply.aso");
        types.insert("imp", "application/vnd.accpac.simply.imp");
        types.insert("twd", "application/vnd.simtech-mindmapper");
        types.insert("csp", "application/vnd.commonspace");
        types.insert("saf", "application/vnd.yamaha.smaf-audio");
        types.insert("mmf", "application/vnd.smaf");
        types.insert("spf", "application/vnd.yamaha.smaf-phrase");
        types.insert("teacher", "application/vnd.smart.teacher");
        types.insert("svd", "application/vnd.svd");
        types.insert("rq", "application/sparql-query");
        types.insert("srx", "application/sparql-results+xml");
        types.insert("gram", "application/srgs");
        types.insert("grxml", "application/srgs+xml");
        types.insert("ssml", "application/ssml+xml");
        types.insert("skp", "application/vnd.koan");
        types.insert("sgml", "text/sgml");
        types.insert("sdc", "application/vnd.stardivision.calc");
        types.insert("sda", "application/vnd.stardivision.draw");
        types.insert("sdd", "application/vnd.stardivision.impress");
        types.insert("smf", "application/vnd.stardivision.math");
        types.insert("sdw", "application/vnd.stardivision.writer");
        types.insert("sgl", "application/vnd.stardivision.writer-global");
        types.insert("sm", "application/vnd.stepmania.stepchart");
        types.insert("sit", "application/x-stuffit");
        types.insert("sitx", "application/x-stuffitx");
        types.insert("sdkm", "application/vnd.solent.sdkm+xml");
        types.insert("xo", "application/vnd.olpc-sugar");
        types.insert("au", "audio/basic");
        types.insert("wqd", "application/vnd.wqd");
        types.insert("sis", "application/vnd.symbian.install");
        types.insert("smi", "application/smil+xml");
        types.insert("xsm", "application/vnd.syncml+xml");
        types.insert("bdm", "application/vnd.syncml.dm+wbxml");
        types.insert("xdm", "application/vnd.syncml.dm+xml");
        types.insert("sv4cpio", "application/x-sv4cpio");
        types.insert("sv4crc", "application/x-sv4crc");
        types.insert("sbml", "application/sbml+xml");
        types.insert("tsv", "text/tab-separated-values");
        types.insert("tiff", "image/tiff");
        types.insert("tao", "application/vnd.tao.intent-module-archive");
        types.insert("tar", "application/x-tar");
        types.insert("tcl", "application/x-tcl");
        types.insert("tex", "application/x-tex");
        types.insert("tfm", "application/x-tex-tfm");
        types.insert("tei", "application/tei+xml");
        types.insert("txt", "text/plain");
        types.insert("dxp", "application/vnd.spotfire.dxp");
        types.insert("sfs", "application/vnd.spotfire.sfs");
        types.insert("tsd", "application/timestamped-data");
        types.insert("tpt", "application/vnd.trid.tpt");
        types.insert("mxs", "application/vnd.triscape.mxs");
        types.insert("t", "text/troff");
        types.insert("tra", "application/vnd.trueapp");
        types.insert("ttf", "application/x-font-ttf");
        types.insert("ttl", "text/turtle");
        types.insert("umj", "application/vnd.umajin");
        types.insert("uoml", "application/vnd.uoml+xml");
        types.insert("unityweb", "application/vnd.unity");
        types.insert("ufd", "application/vnd.ufdl");
        types.insert("uri", "text/uri-list");
        types.insert("utz", "application/vnd.uiq.theme");
        types.insert("ustar", "application/x-ustar");
        types.insert("uu", "text/x-uuencode");
        types.insert("vcs", "text/x-vcalendar");
        types.insert("vcf", "text/x-vcard");
        types.insert("vcd", "application/x-cdlink");
        types.insert("vsf", "application/vnd.vsf");
        types.insert("wrl", "model/vrml");
        types.insert("vcx", "application/vnd.vcx");
        types.insert("mts", "model/vnd.mts");
        types.insert("vtu", "model/vnd.vtu");
        types.insert("vis", "application/vnd.visionary");
        types.insert("viv", "video/vnd.vivo");
        types.insert("ccxml", "application/ccxml+xml,");
        types.insert("vxml", "application/voicexml+xml");
        types.insert("src", "application/x-wais-source");
        types.insert("wbxml", "application/vnd.wap.wbxml");
        types.insert("wbmp", "image/vnd.wap.wbmp");
        types.insert("wav", "audio/x-wav");
        types.insert("davmount", "application/davmount+xml");
        types.insert("woff", "application/x-font-woff");
        types.insert("wspolicy", "application/wspolicy+xml");
        types.insert("webp", "image/webp");
        types.insert("wtb", "application/vnd.webturbo");
        types.insert("wgt", "application/widget");
        types.insert("hlp", "application/winhlp");
        types.insert("wml", "text/vnd.wap.wml");
        types.insert("wmls", "text/vnd.wap.wmlscript");
        types.insert("wmlsc", "application/vnd.wap.wmlscriptc");
        types.insert("wpd", "application/vnd.wordperfect");
        types.insert("stf", "application/vnd.wt.stf");
        types.insert("wsdl", "application/wsdl+xml");
        types.insert("xbm", "image/x-xbitmap");
        types.insert("xpm", "image/x-xpixmap");
        types.insert("xwd", "image/x-xwindowdump");
        types.insert("der", "application/x-x509-ca-cert");
        types.insert("fig", "application/x-xfig");
        types.insert("xhtml", "application/xhtml+xml");
        types.insert("xml", "application/xml");
        types.insert("xdf", "application/xcap-diff+xml");
        types.insert("xenc", "application/xenc+xml");
        types.insert("xer", "application/patch-ops-error+xml");
        types.insert("rl", "application/resource-lists+xml");
        types.insert("rs", "application/rls-services+xml");
        types.insert("rld", "application/resource-lists-diff+xml");
        types.insert("xslt", "application/xslt+xml");
        types.insert("xop", "application/xop+xml");
        types.insert("xpi", "application/x-xpinstall");
        types.insert("xspf", "application/xspf+xml");
        types.insert("xul", "application/vnd.mozilla.xul+xml");
        types.insert("xyz", "chemical/x-xyz");
        types.insert("yaml", "text/yaml");
        types.insert("yang", "application/yang");
        types.insert("yin", "application/yin+xml");
        types.insert("zir", "application/vnd.zul");
        types.insert("zip", "application/zip");
        types.insert("zmm", "application/vnd.handheld-entertainment+xml");
        types.insert("zaz", "application/vnd.zzazz.deck+xml");

        match types.get(format) {
            Some(content_type) => content_type,
            None => "text/plain",
        }
    }
}

fn parse_headers(request: String) -> HashMap<String, String> {
    // println!("{}", request);
    // let request_regex = Regex::new(r"\A(.+)$").unwrap();
    // let header_regex = Regex::new(r"").unwrap;
    let mut headers = HashMap::new();
    let request = request.replace("\0", "");
    for item in request.split("\n") {
        match parse_header(item.trim().to_string()) {
            Some(header) => {
                headers.insert(header.name, header.value);
            },
            None => {},
        };
    }
    let parts: Vec<&str> = request.split("\n").collect();

    match parts.first() {
        Some(method_string) => {
            headers.insert("Method".to_string(), parse_method(method_string));
            headers.insert("Path".to_string(), parse_path(method_string));
            headers.insert("Version".to_string(), parse_version(method_string));
        },
        None => {},
    };
    // println!("{:?}", headers);
    headers
}

fn parse_method(request: &str) -> String {
    let parts: Vec<&str> = request.split(' ').collect();
    match parts.first() {
        Some(method) => method.to_string(),
        None => "GET".to_string(),
    }
}

fn parse_path(request: &str) -> String {
    let method = parse_method(request);
    let request_without_method = request.to_string().replace(&method, "");
    let request_without_method = request_without_method.trim();
    // println!("{:?}", request_without_method);
    let parts: Vec<&str> = request_without_method.split(' ').collect();

    match parts.first() {
        Some(path) => path.to_string(),
        None => "/".to_string(),
    }
}

fn parse_version(request: &str) -> String {
    let method = parse_method(request);
    let path = parse_path(request);
    let version_string = request.to_string().replace(&method, "");
    let version_string = match path.as_ref() {
        "/" => {
            version_string.trim().trim_left_matches('/').to_string()
        },
        _ => {
            version_string.replace(&path, "").to_string()
        }
    };
    version_string.trim().to_string()
}

fn parse_header(header: String) -> Option<Header> {
    // let header_cleaned = header.replace("\0", "");
    let parts: Vec<&str> = header.split(": ").collect();
    // let tail = parts.tail();
    // println!("{:?}", head);
    let header_name = match parts.first() {
        Some(header_name) => header_name.trim(),
        None => "",
    };
    // let header_name = format!("{}", header_name.trim());

    let header_value = format!("{}", header.replace(&format!("{}: ", header_name), "").trim());
    // let header_value = match parts.last() {
    //     Some(header_value) => header_value.trim(),
    //     None => "",
    // };
    let header_name = header_name.to_string();
    let header_value = header_value.to_string();

    if header_name == header_value {
        // let header_value = "";
        return None
    };
    if header_value != "" {
        Some(Header {name: header_name, value: header_value} )
    } else {
        None
    }
}