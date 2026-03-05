// use logos::Logos;
use deriveUtils::{FileScanner};
use commun_utils_handler::{FileScanner,ScanWarn};
// use deriveUtils::FileScanner;

use std::error::Error;

use regex::{Match, Regex};

#[derive(FileScanner)]
pub enum MalwareWarnRaise {
     #[regex(r#"(fetch|XMLHttpRequest|axios|WebSocket|EventSource|navigator\.sendBeacon|postMessage|onmessage|addEventListener[(].message.[)])"#)]
    NetworkAccess,
    
    #[regex(r#"(https?://|wss?://|/?api/?|/?upload/?|/?[A-z0-9\._]\w+\.php/?|/?beacon/?|/?socket/?|callback=|token=|session=|jwt)"#)]
    Exflitration,

    #[regex(r#"(navigator\.(userAgent|platform|hardwareConcurrency|language|plugins)|screen\.(width|height)|devicePixelRatio|Intl\.DateTimeFormat[(][)]\.resolvedOptions[(][)]\.timeZone)"#)]
    SysInfomationCollect,

    #[regex(r#"(canvas\.toDataURL|getContext[(].webgl.[)]|getParameter|WEBGL_debug_renderer_info|UNMASKED_VENDOR_WEBGL|UNMASKED_RENDERER_WEBGL)"#)]
    WebGLInfomationCollect,

    #[regex(r#"(document\.cookie|(local|session)Storage|indexedDB|caches\.keys|FileReader|Blob)"#)]
    NavigatorRessourceAccess,

    #[regex(r#"(WebAssembly\.(instantiate(Streaming)?|compile|Module|Memory|Table))"#)]
    WasmLoading,

    #[regex(r#"(wasm_bindgen|__wbindgen|emscriptenemscripten|env\.memory|memory\.grow|importObject|exports\._)"#)]
    ObfuscationPacking,

    #[regex(r#"(eval[(]|Function[(]|setTimeout[(](["A-z0-9]\w+|[A-z]).|setInterval[(](["A-z0-9]\w+|[A-z]).|unescape[(]|(b|a)to(a|b)|decodeURIComponent[(]|String\.fromCharCode[(]|charCodeAt[(])"#)]
    ObfuscationEncodage,

    #[regex(r#"(0x|\\x|\\u00|(\["..."\])+|!(!|[+])\[\])"#)]
    ObfuscationSensitiveHex,
    
    #[regex(r#"(crypto\.subtle|digest|SHA-1|SHA-256|MD5|AES|HMAC|PBKDF2|randomUUID|getRandomValues)"#)]
    Encoding,

    #[regex(r#"(typeof (window|global)|process\.(env|platform)|require[(].(node:)?(os|fs|child_process).[)]|import ([A-z]|[A-z0-9_\.]\w+) from .(node:)?(os|fs|child_process).|Deno\.env)"#)]
    EnvScrapping,

    #[regex(r"/?(192\.168\.|10\.0\.|172\.16\.|localhost|127\.0\.0\.1|::1|intranet|admin|config|status|health)")]
    InternalScanning,

    #[regex(r"(cmd|shell|exec|spawn|download|encrypt|decrypt|miner|wallet)")]
    SupectKeyWord,
}

// impl TryFrom<&str> for MalwareWarnRaise {
//     type Error = Box<dyn Error>;

//     // fn try_from(value: &str) -> Result<Self, Self::Error> {
//     //     match value {
//     //         x if Regex::new(r#"(fetch|XMLHttpRequest|axios|WebSocket|EventSource|navigator\.sendBeacon|postMessage|onmessage|addEventListener[(].message.[)])"#)?.is_match(x) => {},
//     //         _ => Ok()
//     //     }
//     // }
// }


// #[derive(Logos,Debug,PartialEq)]
// enum MalwareScan {

//     #[regex(r"[ \n\t\f]+", logos::skip)]
//     Ignored,

//     #[regex(r#"(fetch|XMLHttpRequest|axios|WebSocket|EventSource|navigator\.sendBeacon|postMessage|onmessage|addEventListener[(].message.[)])"#)]
//     NetworkAccess,
    
//     #[regex(r#"(https?://|wss?://|/?api/?|/?upload/?|/?[A-z0-9\._]\w+\.php/?|/?beacon/?|/?socket/?|callback=|token=|session=|jwt)"#)]
//     Exflitration,

//     #[regex(r#"(navigator\.(userAgent|platform|hardwareConcurrency|language|plugins)|screen\.(width|height)|devicePixelRatio|Intl\.DateTimeFormat[(][)]\.resolvedOptions[(][)]\.timeZone)"#)]
//     SysInfomationCollect,

//     #[regex(r#"(canvas\.toDataURL|getContext[(].webgl.[)]|getParameter|WEBGL_debug_renderer_info|UNMASKED_VENDOR_WEBGL|UNMASKED_RENDERER_WEBGL)"#)]
//     WebGLInfomationCollect,

//     #[regex(r#"(document\.cookie|(local|session)Storage|indexedDB|caches\.keys|FileReader|Blob)"#)]
//     NavigatorRessourceAccess,

//     #[regex(r#"(WebAssembly\.(instantiate(Streaming)?|compile|Module|Memory|Table))"#)]
//     WasmLoading,

//     #[regex(r#"(wasm_bindgen|__wbindgen|emscriptenemscripten|env\.memory|memory\.grow|importObject|exports\._)"#)]
//     ObfuscationPacking,

//     #[regex(r#"(eval[(]|Function[(]|setTimeout[(](["A-z0-9]\w+|[A-z]).|setInterval[(](["A-z0-9]\w+|[A-z]).|unescape[(]|(b|a)to(a|b)|decodeURIComponent[(]|String\.fromCharCode[(]|charCodeAt[(])"#)]
//     ObfuscationEncodage,

//     #[regex(r#"(0x|\\x|\\u00|(\["..."\])+|!(!|[+])\[\])"#)]
//     ObfuscationSensitiveHex,
    
//     #[regex(r#"(crypto\.subtle|digest|SHA-1|SHA-256|MD5|AES|HMAC|PBKDF2|randomUUID|getRandomValues)"#)]
//     Encoding,

//     #[regex(r#"(typeof (window|global)|process\.(env|platform)|require[(].(node:)?(os|fs|child_process).[)]|import ([A-z]|[A-z0-9_\.]\w+) from .(node:)?(os|fs|child_process).|Deno\.env)"#)]
//     EnvScrapping,

//     #[regex(r"/?(192\.168\.|10\.0\.|172\.16\.|localhost|127\.0\.0\.1|::1|intranet|admin|config|status|health)")]
//     InternalScanning,

//     #[regex(r"(cmd|shell|exec|spawn|download|encrypt|decrypt|miner|wallet)")]
//     SupectKeyWord,

// }
