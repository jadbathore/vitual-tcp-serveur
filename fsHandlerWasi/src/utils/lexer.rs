// use logos::Logos;
use deriveUtils::{FileScanner};
use commun_utils_handler::{FileScanner,ScanBytesSubject};


#[derive(FileScanner)]
pub enum MalwareWarnRaiseApp {
     #[regex(r#"fetch|XMLHttpRequest|axios|WebSocket|EventSource|navigator\.sendBeacon|postMessage|onmessage|addEventListener[(].message.[)]"#)]
    NetworkAccess,
    
    #[regex(r#"https?://|wss?://|/?api/?|/?upload/?|/?[A-z0-9\._]\w+\.php/?|/?beacon/?|/?socket/?|callback=|token=|session=|jwt"#)]
    Exflitration,

    #[regex(r#"navigator\.(userAgent|platform|hardwareConcurrency|language|plugins)|screen\.(width|height)|devicePixelRatio|Intl\.DateTimeFormat[(][)]\.resolvedOptions[(][)]\.timeZone"#)]
    SysInfomationCollect,

    #[regex(r#"canvas\.toDataURL|getContext[(].webgl.[)]|getParameter|WEBGL_debug_renderer_info|UNMASKED_VENDOR_WEBGL|UNMASKED_RENDERER_WEBGL"#)]
    WebGLInfomationCollect,

    #[regex(r#"document\.cookie|(local|session)Storage|indexedDB|caches\.keys|FileReader|Blob"#)]
    NavigatorRessourceAccess,

    #[regex(r#"WebAssembly\.(instantiate(Streaming)?|compile|Module|Memory|Table)"#)]
    WasmLoading,

    #[regex(r#"wasm_bindgen|__wbindgen|emscriptenemscripten|env\.memory|memory\.grow|importObject|exports\._"#)]
    ObfuscationPacking,

    #[regex(r#"eval[(]|Function[(]|setTimeout[(](["A-z0-9]\w+|[A-z]).|setInterval[(](["A-z0-9]\w+|[A-z]).|unescape[(]|(b|a)to(a|b)|decodeURIComponent[(]|String\.fromCharCode[(]|charCodeAt[(]"#)]
    ObfuscationEncodage,

    #[regex(r#"(0x|\\x|\\u00|(\["..."\])+|!(!|[+])\[\])"#)]
    ObfuscationSensitiveHex,
    
    #[regex(r#"crypto\.subtle|digest|SHA-1|SHA-256|MD5|AES|HMAC|PBKDF2|randomUUID|getRandomValues"#)]
    Encoding,

    #[regex(r#"typeof\s+(window|global)|process\.(env|platform)|require[(].(node:)?(os|fs|child_process).[)]|import ([A-z]|[A-z0-9_\.]\w+) from .(node:)?(os|fs|child_process).|Deno\.env"#)]
    EnvScrapping,

    #[regex(r"/?192\.168\.|10\.0\.|172\.16\.|localhost|127\.0\.0\.1|::1|intranet|admin|config|status|health")]
    InternalScanning,

    #[regex(r"cmd|shell|exec|spawn|download|encrypt|decrypt|miner|wallet")]
    SupectKeyWord,
}
#[derive(FileScanner)]
pub enum MalwareWarnRaiseImg {
    #[regex(r";|\{|\}|\[|\]|\(|\)|<|>|\$|\%|\^|\&|\*|\=|\\|\/|`|\~")]
    ASCII,
    
    #[regex(r";|&&|\|\||>>?|<<?|\$[(][)]|\`")]
    ASCIIShell,

    #[regex(r"<\?php|<script|</script|<\?=|\?>")]
    ScriptWeb,

    #[regex(r"cmd\.exe|powershell|bash|sh|wget|curl|nc|netcat|whoami|chmod")]
    Token,

    #[regex(r"CreateProcess|VirtualAlloc|WriteProcessMemory|LoadLibrary|GetProcAddress|WinExec")]
    ApiWindow,

    #[regex(r"shellcode|payload|backdoor|reverse|connect|inject|exploit")]
    Payload,

    #[regex(r"[A-Za-z0-9+/]{100,}=")]
    Base64,

    #[regex(r"[0-9A-Fa-f]{200,}")]
    Hexadecimal,

    #[regex(r"eval[(]|atob[(]|document\.|window\.|function[(]")]
    Javascript,

    #[regex(r"<\?php|eval[(]|base64\_decode[(]")]
    Php,

    #[regex(r"import\s+os|import\s+subprocess|exec[(]")]
    Python,

    #[regex(r"[A-z0-9]\w+(\.(php|(c|m)?js|exe|so|rlib))?(\\x00)+\.(jpe?g|png)")]
    ByPassParser,
    
    #[regex(r"(\\x31\\xC0\\x50\\x68|\\x2f\\x62\\x69\\x6e)(\\x2F){1,2}(\\x73\\x68|\\x62\\x61\\x73\\x68)\\x00")]
    ShellCodeAccess,

    #[regex(r"(\\x00){30,}")]
    AbnormalNumberOfNull,

    #[regex(r"MZ|PE|ELF|PK")]
    MagicByte,

    #[regex(r"(?i)(Comment\sExtension|Application\sExtension)")]
    SuspectGifExt
}