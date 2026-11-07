//! `http <METHOD> <url>`: a request outside the page, through `curl`, which
//! every supported OS ships, so the binary carries no TLS stack. curl reads
//! its options (URL, headers, body) on stdin, so a secret in a header or the
//! body never reaches the process list.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub status: u16,
    /// The last answer's headers.
    pub headers: Vec<(String, String)>,
    /// `Set-Cookie`s of every answer on the way, redirects included, each
    /// with the URL of the answer that set it.
    pub cookies: Vec<(String, String)>,
    pub body: String,
    /// The whole exchange, redirects included, as curl timed it (ms).
    pub ms: f64,
}

/// What an `http` step sends.
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    /// As given (`--json`, `--body`, a file's text).
    Raw(String),
    /// `--form name=value`…: `application/x-www-form-urlencoded`.
    Form(Vec<(String, String)>),
    /// `--multipart name=value` or `name=@file`…: `multipart/form-data`.
    Multipart(Vec<(String, String)>),
}

/// Whether to follow redirects, and to accept any certificate.
#[derive(Debug, Clone, Copy)]
pub struct How {
    pub follow: bool,
    pub insecure: bool,
}

/// A curl config string: `"…"` with `\` and `"` escaped, and the control
/// characters curl unescapes.
fn quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What curl writes after the answer: how long the exchange took. The body
/// before it is the answer's own.
const TIMED: &str = "\n--primavera-time ";

/// curl's config for one request.
fn config(method: &str, url: &str, headers: &[String], body: Option<&Body>, limit: Duration, how: How) -> String {
    let mut c = format!("url = {}\nmax-time = {:.3}\n", quoted(url), limit.as_secs_f64());
    c.push_str(&format!("write-out = {}\n", quoted(&format!("{TIMED}%{{time_total}}"))));
    if how.follow {
        // A 301/302/303 after a POST is fetched with GET, as browsers do;
        // a cookie set on the way goes to the next hop.
        c.push_str("location\nmax-redirs = 20\ncookie = \"\"\n");
    }
    if how.insecure {
        c.push_str("insecure\n");
    }
    if method == "HEAD" {
        c.push_str("head\n");
    } else {
        c.push_str(&format!("request = {}\n", quoted(method)));
    }
    for h in headers {
        c.push_str(&format!("header = {}\n", quoted(h)));
    }
    let has = |name: &str| has_header(headers, name);
    match body {
        Some(Body::Raw(b)) => c.push_str(&format!("data-binary = {}\n", quoted(b))),
        Some(Body::Form(fields)) => {
            if !has("content-type") {
                c.push_str("header = \"Content-Type: application/x-www-form-urlencoded\"\n");
            }
            c.push_str(&format!("data-binary = {}\n", quoted(&form_encoded(fields))));
        }
        Some(Body::Multipart(parts)) => {
            for (name, value) in parts {
                // `@file` uploads it (curl reads `;type=` after it); any
                // other value is sent as it is written.
                match value.strip_prefix('@') {
                    Some(_) => c.push_str(&format!("form = {}\n", quoted(&format!("{name}={value}")))),
                    None => c.push_str(&format!("form-string = {}\n", quoted(&format!("{name}={value}")))),
                }
            }
        }
        None => {}
    }
    c
}

/// Whether `Name: value` lines have one called `name` (any case).
pub fn has_header(headers: &[String], name: &str) -> bool {
    headers.iter().any(|h| h.split(':').next().is_some_and(|k| k.trim().eq_ignore_ascii_case(name)))
}

/// `application/x-www-form-urlencoded`, as a browser sends a form.
pub fn form_encoded(fields: &[(String, String)]) -> String {
    let enc = |s: &str| {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => out.push(b as char),
                b' ' => out.push('+'),
                b => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    };
    fields.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect::<Vec<_>>().join("&")
}

/// `url` with `name=value` pairs added to its query, percent-encoded (a
/// `#fragment` stays last).
pub fn with_query(url: &str, pairs: &[(String, String)]) -> String {
    if pairs.is_empty() {
        return url.to_string();
    }
    let (rest, frag) = url.split_once('#').map_or((url, None), |(u, f)| (u, Some(f)));
    let enc = |s: &str| {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
                b => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    };
    let q = pairs.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect::<Vec<_>>().join("&");
    let sep = if !rest.contains('?') {
        "?"
    } else if rest.ends_with('?') || rest.ends_with('&') {
        ""
    } else {
        "&"
    };
    match frag {
        Some(f) => format!("{rest}{sep}{q}#{f}"),
        None => format!("{rest}{sep}{q}"),
    }
}

/// curl `--include` output: the last header block (after any `100
/// Continue` and the redirects it followed), every `Set-Cookie` on the way
/// with the URL of the answer that set it, the body, and the time curl
/// wrote after it.
fn parse_response(raw: &[u8], url: &str) -> Result<Response, String> {
    let (mut rest, ms) = match raw.windows(TIMED.len()).rposition(|w| w == TIMED.as_bytes()) {
        Some(i) => {
            let secs: f64 = String::from_utf8_lossy(&raw[i + TIMED.len()..]).trim().parse().unwrap_or(0.0);
            (&raw[..i], secs * 1000.0)
        }
        None => (raw, 0.0),
    };
    let mut cookies = Vec::new();
    let mut at = url.to_string();
    loop {
        let end = rest
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|i| (i, 4))
            .or_else(|| rest.windows(2).position(|w| w == b"\n\n").map(|i| (i, 2)))
            .or_else(|| {
                // A HEAD answer, or one with no body: the headers end the output.
                let t = String::from_utf8_lossy(rest);
                (t.starts_with("HTTP/") && (t.ends_with("\r\n") || t.ends_with('\n'))).then_some((rest.len(), 0))
            })
            .ok_or("no HTTP response")?;
        let head = String::from_utf8_lossy(&rest[..end.0]).into_owned();
        let body = &rest[(end.0 + end.1).min(rest.len())..];
        let mut lines = head.lines();
        let status_line = lines.next().unwrap_or("");
        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("not an HTTP status line: {status_line:?}"))?;
        let headers: Vec<(String, String)> = lines
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();
        cookies.extend(
            headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie")).map(|(_, v)| (v.clone(), at.clone())),
        );
        // An interim answer or a followed redirect, then the real one.
        if ((100..200).contains(&status) || (300..400).contains(&status)) && body.starts_with(b"HTTP/") {
            if (300..400).contains(&status) {
                if let Some((_, loc)) = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("location")) {
                    at = resolve(&at, loc);
                }
            }
            rest = body;
            continue;
        }
        return Ok(Response { status, headers, cookies, body: String::from_utf8_lossy(body).into_owned(), ms });
    }
}

/// A `Location` against the URL it answered.
pub fn resolve(base: &str, loc: &str) -> String {
    if loc.contains("://") {
        return loc.to_string();
    }
    let Some(u) = Url::parse(base) else { return loc.to_string() };
    let origin = match u.port {
        Some(p) => format!("{}://{}:{p}", u.scheme, u.host_raw),
        None => format!("{}://{}", u.scheme, u.host_raw),
    };
    if let Some(rest) = loc.strip_prefix("//") {
        return format!("{}://{rest}", u.scheme);
    }
    if loc.starts_with('/') {
        return format!("{origin}{loc}");
    }
    if loc.starts_with('?') {
        return format!("{origin}{}{loc}", u.path);
    }
    let dir = &u.path[..u.path.rfind('/').map_or(0, |i| i + 1)];
    format!("{origin}{}{loc}", if dir.is_empty() { "/" } else { dir })
}

/// The parts of a URL a cookie cares about.
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    pub scheme: String,
    /// Lowercase, without brackets or port.
    pub host: String,
    /// As written (an IPv6 address keeps its brackets).
    host_raw: String,
    pub port: Option<u16>,
    /// The path, without query or fragment (`/` when none).
    pub path: String,
}

impl Url {
    pub fn parse(url: &str) -> Option<Url> {
        let (scheme, rest) = url.split_once("://")?;
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let (authority, tail) = rest.split_at(end);
        let hostport = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
        let (host_raw, port) = if let Some(v6) = hostport.strip_prefix('[') {
            let close = v6.find(']')?;
            let port = v6[close + 1..].strip_prefix(':').and_then(|p| p.parse().ok());
            (format!("[{}]", &v6[..close]), port)
        } else {
            match hostport.rsplit_once(':') {
                Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() => {
                    (h.to_string(), p.parse().ok())
                }
                _ => (hostport.to_string(), None),
            }
        };
        let path = tail.split(['?', '#']).next().unwrap_or("");
        Some(Url {
            scheme: scheme.to_ascii_lowercase(),
            host: host_raw.trim_start_matches('[').trim_end_matches(']').to_ascii_lowercase(),
            host_raw,
            port,
            path: if path.is_empty() { "/".into() } else { path.to_string() },
        })
    }
}

/// Make the request. `headers` are `Name: value` lines.
pub fn request(
    method: &str,
    url: &str,
    headers: &[String],
    body: Option<&Body>,
    limit: Duration,
    how: How,
) -> Result<Response, String> {
    let mut child = Command::new("curl")
        .args(["-sS", "--include", "--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("http needs curl on the PATH: {e}"))?;
    let cfg = config(method, url, headers, body, limit, how);
    child.stdin.take().expect("piped").write_all(cfg.as_bytes()).map_err(|e| format!("curl: {e}"))?;
    let out = child.wait_with_output().map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        let why = String::from_utf8_lossy(&out.stderr);
        let why = why.trim().trim_start_matches("curl: ");
        return Err(format!("{method} {url}: {}", if why.is_empty() { "curl failed" } else { why }));
    }
    parse_response(&out.stdout, url)
}

/// Days since 1970-01-01 of a civil date.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// An HTTP date (`Wed, 21 Oct 2015 07:28:00 GMT`, or with dashes) in Unix
/// seconds.
fn http_date(s: &str) -> Option<i64> {
    const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let parts: Vec<&str> = s.split([' ', '-', ':', ',']).filter(|p| !p.is_empty()).collect();
    // [weekday,] day month year h m s [GMT]
    let at = parts.iter().position(|p| p.chars().all(|c| c.is_ascii_digit()))?;
    let (d, mon, y, h, mi, se) = (
        parts.get(at)?,
        parts.get(at + 1)?,
        parts.get(at + 2)?,
        parts.get(at + 3)?,
        parts.get(at + 4)?,
        parts.get(at + 5)?,
    );
    let m = MONTHS.iter().position(|x| mon.to_ascii_lowercase().starts_with(x))? as i64 + 1;
    let mut y: i64 = y.parse().ok()?;
    if y < 100 {
        y += if y < 70 { 2000 } else { 1900 };
    }
    let days = days_from_civil(y, m, d.parse().ok()?);
    Some(days * 86_400 + h.parse::<i64>().ok()? * 3600 + mi.parse::<i64>().ok()? * 60 + se.parse::<i64>().ok()?)
}

/// A `Set-Cookie` header, read.
#[derive(Debug, Clone, PartialEq, Default)]
struct SetCookie {
    name: String,
    value: String,
    /// The `Domain` attribute, lowercase, without its leading dot.
    domain: Option<String>,
    path: Option<String>,
    /// When it expires (Unix s): `Max-Age` wins over `Expires`.
    until: Option<i64>,
    http_only: bool,
    secure: bool,
    same_site: Option<String>,
}

impl SetCookie {
    fn parse(set_cookie: &str, now: i64) -> Option<SetCookie> {
        let mut parts = set_cookie.split(';');
        let (name, value) = parts.next()?.split_once('=')?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return None;
        }
        let mut c = SetCookie { name, value: value.trim().to_string(), ..Default::default() };
        let (mut expires, mut max_age) = (None, None);
        for attr in parts {
            let (k, v) = attr.split_once('=').map(|(k, v)| (k.trim(), v.trim())).unwrap_or((attr.trim(), ""));
            match k.to_ascii_lowercase().as_str() {
                "domain" if !v.is_empty() => c.domain = Some(v.trim_start_matches('.').to_ascii_lowercase()),
                "path" if v.starts_with('/') => c.path = Some(v.to_string()),
                "expires" => expires = http_date(v),
                "max-age" => max_age = v.parse::<i64>().ok(),
                "httponly" => c.http_only = true,
                "secure" => c.secure = true,
                "samesite" if !v.is_empty() => {
                    let mut s = v.to_ascii_lowercase();
                    s[..1].make_ascii_uppercase();
                    c.same_site = Some(s);
                }
                _ => {}
            }
        }
        c.until = max_age.map(|a| now + a).or(expires);
        Some(c)
    }
}

/// A `Set-Cookie` value as navigera `cookies` words for the scenario's
/// browser: `set <name> <value> …`, or `delete <name> …` for one that
/// expires it.
pub fn cookie_words(set_cookie: &str, url: &str, now: i64) -> Option<Vec<String>> {
    let c = SetCookie::parse(set_cookie, now)?;
    let value = c.value.trim_matches('"').to_string();
    let mut words = match c.until {
        Some(t) if t <= now => vec!["cookies".to_string(), "delete".into(), c.name],
        _ => vec!["cookies".to_string(), "set".into(), c.name, value],
    };
    match c.domain {
        Some(d) => words.extend(["--domain".to_string(), d]),
        None => words.extend(["--url".to_string(), url.to_string()]),
    }
    if words[1] == "set" {
        if let Some(p) = c.path {
            words.extend(["--path".to_string(), p]);
        }
        if let Some(t) = c.until {
            words.extend(["--expires".to_string(), t.to_string()]);
        }
        if c.http_only {
            words.push("--http-only".to_string());
        }
        if c.secure {
            words.push("--secure".to_string());
        }
        if let Some(s) = c.same_site {
            words.extend(["--same-site".to_string(), s]);
        }
    }
    Some(words)
}

/// One cookie a scenario without a browser holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    /// Lowercase, no leading dot.
    pub domain: String,
    /// Set without a `Domain`: sent to that host only, not its subdomains.
    pub host_only: bool,
    pub path: String,
    /// Unix seconds; `None` for a session cookie.
    pub expires: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

/// The cookies of a scenario that runs with no browser: what its `http`
/// answers set, sent back with its later requests the way a browser would
/// (RFC 6265: domain, path, `Secure`, expiry), and saved in navigera's
/// state format (`state save`) so a browser can load them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Jar {
    cookies: Vec<Cookie>,
}

fn is_ip(host: &str) -> bool {
    host.parse::<std::net::IpAddr>().is_ok()
}

/// A secure context, as Chrome counts one for cookies: https, or this machine.
fn secure_context(u: &Url) -> bool {
    u.scheme == "https"
        || matches!(u.host.as_str(), "localhost" | "127.0.0.1" | "::1")
        || u.host.ends_with(".localhost")
}

fn domain_match(host: &str, domain: &str) -> bool {
    host == domain || (host.ends_with(domain) && host[..host.len() - domain.len()].ends_with('.') && !is_ip(host))
}

fn path_match(req: &str, cookie: &str) -> bool {
    req == cookie || (req.starts_with(cookie) && (cookie.ends_with('/') || req[cookie.len()..].starts_with('/')))
}

/// RFC 6265's default path: the request path up to its last `/`.
fn default_path(path: &str) -> String {
    match path.rfind('/') {
        Some(0) | None => "/".into(),
        Some(i) => path[..i].to_string(),
    }
}

impl Jar {
    /// Take a `Set-Cookie` the answer from `url` sent.
    pub fn store(&mut self, set_cookie: &str, url: &str, now: i64) {
        let (Some(c), Some(u)) = (SetCookie::parse(set_cookie, now), Url::parse(url)) else { return };
        let (domain, host_only) = match c.domain {
            // A domain it may not set (another site's, or a parent of an IP).
            Some(d) if !domain_match(&u.host, &d) => return,
            Some(d) => (d, false),
            None => (u.host.clone(), true),
        };
        if c.secure && !secure_context(&u) {
            return;
        }
        let path = c.path.unwrap_or_else(|| default_path(&u.path));
        self.cookies.retain(|k| !(k.name == c.name && k.domain == domain && k.path == path));
        if c.until.is_some_and(|t| t <= now) {
            return;
        }
        self.cookies.push(Cookie {
            name: c.name,
            value: c.value,
            domain,
            host_only,
            path,
            expires: c.until,
            secure: c.secure,
            http_only: c.http_only,
            same_site: c.same_site,
        });
    }

    /// The cookies a request to `url` carries, longest path first.
    pub fn matching(&self, url: &str, now: i64) -> Vec<&Cookie> {
        let Some(u) = Url::parse(url) else { return Vec::new() };
        let mut out: Vec<&Cookie> = self
            .cookies
            .iter()
            .filter(|c| c.expires.is_none_or(|t| t > now))
            .filter(|c| if c.host_only { u.host == c.domain } else { domain_match(&u.host, &c.domain) })
            .filter(|c| path_match(&u.path, &c.path))
            .filter(|c| !c.secure || secure_context(&u))
            .collect();
        // Stable: equal paths keep the order they were set in.
        out.sort_by_key(|c| std::cmp::Reverse(c.path.len()));
        out
    }

    /// The `Cookie` header for a request to `url`, if any cookie goes.
    pub fn header(&self, url: &str, now: i64) -> Option<String> {
        let pairs: Vec<String> = self.matching(url, now).iter().map(|c| format!("{}={}", c.name, c.value)).collect();
        (!pairs.is_empty()).then(|| pairs.join("; "))
    }

    /// The latest cookie of that name that hasn't expired, any domain.
    pub fn get(&self, name: &str, now: i64) -> Option<&Cookie> {
        self.cookies.iter().rev().find(|c| c.name == name && c.expires.is_none_or(|t| t > now))
    }

    /// As navigera's `state save` writes it (`{cookies, origins}`), so a
    /// browser scenario can start from it.
    pub fn state(&self, now: i64) -> serde_json::Value {
        let cookies: Vec<serde_json::Value> = self
            .cookies
            .iter()
            .filter(|c| c.expires.is_none_or(|t| t > now))
            .map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "value": c.value,
                    "domain": if c.host_only { c.domain.clone() } else { format!(".{}", c.domain) },
                    "path": c.path,
                    "expires": c.expires.map_or(-1.0, |t| t as f64),
                    "httpOnly": c.http_only,
                    "secure": c.secure,
                    "sameSite": c.same_site.clone().unwrap_or_else(|| "Lax".into()),
                })
            })
            .collect();
        serde_json::json!({ "cookies": cookies, "origins": [] })
    }

    /// The cookies of a `state save` file (a browser's or a jar's).
    pub fn load(&mut self, state: &serde_json::Value, now: i64) {
        for c in state.get("cookies").and_then(|c| c.as_array()).into_iter().flatten() {
            let (Some(name), Some(value), Some(domain)) =
                (c["name"].as_str(), c["value"].as_str(), c["domain"].as_str())
            else {
                continue;
            };
            let expires = c["expires"].as_f64().filter(|e| *e > 0.0).map(|e| e as i64);
            if expires.is_some_and(|t| t <= now) {
                continue;
            }
            let host_only = !domain.starts_with('.');
            let domain = domain.trim_start_matches('.').to_ascii_lowercase();
            let path = c["path"].as_str().unwrap_or("/").to_string();
            self.cookies.retain(|k| !(k.name == name && k.domain == domain && k.path == path));
            self.cookies.push(Cookie {
                name: name.to_string(),
                value: value.to_string(),
                domain,
                host_only,
                path,
                expires,
                secure: c["secure"].as_bool().unwrap_or(false),
                http_only: c["httpOnly"].as_bool().unwrap_or(false),
                same_site: c["sameSite"].as_str().map(String::from),
            });
        }
    }
}

/// Unix seconds now.
pub fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_quotes_what_curl_unquotes() {
        let plain = How { follow: false, insecure: false };
        let c = config(
            "POST",
            "http://h/a",
            &["X-Note: say \"hi\"\\".into()],
            Some(&Body::Raw("{\"a\":\n1}".into())),
            Duration::from_millis(1500),
            plain,
        );
        assert_eq!(
            c,
            "url = \"http://h/a\"\nmax-time = 1.500\nwrite-out = \"\\n--primavera-time %{time_total}\"\nrequest = \"POST\"\nheader = \"X-Note: say \\\"hi\\\"\\\\\"\ndata-binary = \"{\\\"a\\\":\\n1}\"\n"
        );
        assert!(config("HEAD", "http://h/", &[], None, Duration::from_secs(1), plain).contains("\nhead\n"));
        let both = config("GET", "https://h/", &[], None, Duration::from_secs(1), How { follow: true, insecure: true });
        assert!(both.contains("\nlocation\nmax-redirs = 20\ncookie = \"\"\ninsecure\n"), "{both}");
    }

    #[test]
    fn responses_skip_interim_answers() {
        let raw = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 201 Created\r\nContent-Type: application/json\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\n\r\n{\"id\":7}";
        let r = parse_response(raw, "http://h/").unwrap();
        assert_eq!(r.status, 201);
        assert_eq!(r.body, "{\"id\":7}");
        assert_eq!(r.headers[0], ("Content-Type".to_string(), "application/json".to_string()));
        assert_eq!(r.headers.iter().filter(|(k, _)| k == "Set-Cookie").count(), 2);
        assert_eq!(parse_response(b"HTTP/2 204\r\n\r\n", "http://h/").unwrap().status, 204);
        assert!(parse_response(b"garbage", "http://h/").is_err());
        // Followed redirects: the last answer, with the cookies set on the way.
        let raw = b"HTTP/1.1 302 Found\r\nLocation: /home\r\nSet-Cookie: sid=1\r\n\r\nHTTP/1.1 200 OK\r\nSet-Cookie: seen=1\r\n\r\nhome";
        let r = parse_response(raw, "http://h/").unwrap();
        assert_eq!((r.status, r.body.as_str()), (200, "home"));
        let set: Vec<(&str, &str)> = r.cookies.iter().map(|(c, u)| (c.as_str(), u.as_str())).collect();
        assert_eq!(set, [("sid=1", "http://h/"), ("seen=1", "http://h/home")]);
        // A redirect not followed is the answer.
        assert_eq!(parse_response(b"HTTP/1.1 302 Found\r\nLocation: /x\r\n\r\n", "http://h/").unwrap().status, 302);
    }

    #[test]
    fn set_cookie_becomes_navigera_words() {
        let now = 1_700_000_000;
        assert_eq!(
            cookie_words("sid=abc; Path=/; HttpOnly; SameSite=lax", "http://h/api", now).unwrap().join(" "),
            "cookies set sid abc --url http://h/api --path / --http-only --same-site Lax"
        );
        assert_eq!(
            cookie_words("t=1; Domain=.shop.test; Max-Age=60; Secure", "https://shop.test/", now).unwrap().join(" "),
            format!("cookies set t 1 --domain shop.test --expires {} --secure", now + 60)
        );
        assert_eq!(
            cookie_words("sid=; Expires=Thu, 01 Jan 1970 00:00:00 GMT", "http://h/", now).unwrap().join(" "),
            "cookies delete sid --url http://h/"
        );
        assert_eq!(http_date("Wed, 21 Oct 2015 07:28:00 GMT"), Some(1_445_412_480));
        assert_eq!(http_date("Wed, 21-Oct-15 07:28:00 GMT"), Some(1_445_412_480));
        assert!(cookie_words("novalue", "http://h/", now).is_none());
    }

    #[test]
    fn the_time_curl_writes_after_the_body_is_read_off() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhello\n--primavera-time 0.012500";
        let r = parse_response(raw, "http://h/").unwrap();
        assert_eq!(r.body, "hello");
        assert!((r.ms - 12.5).abs() < 1e-9, "{}", r.ms);
        // HEAD: headers only, then the time.
        let r =
            parse_response(b"HTTP/1.1 204 No Content\r\nX: 1\r\n\r\n\n--primavera-time 0.001", "http://h/").unwrap();
        assert_eq!((r.status, r.body.as_str()), (204, ""));
    }

    #[test]
    fn redirects_resolve_against_the_url_they_answered() {
        assert_eq!(resolve("http://h:8/a/b?q=1", "/c"), "http://h:8/c");
        assert_eq!(resolve("http://h/a/b", "c?x"), "http://h/a/c?x");
        assert_eq!(resolve("https://h/a", "//o.test/z"), "https://o.test/z");
        assert_eq!(resolve("http://h/a", "https://o.test/"), "https://o.test/");
        assert_eq!(resolve("http://[::1]:9/a/b", "?p=2"), "http://[::1]:9/a/b?p=2");
        let u = Url::parse("HTTPS://User@Shop.Test:8443/api/v1?x#y").unwrap();
        assert_eq!(
            (u.scheme.as_str(), u.host.as_str(), u.port, u.path.as_str()),
            ("https", "shop.test", Some(8443), "/api/v1")
        );
        assert_eq!(Url::parse("http://h").unwrap().path, "/");
    }

    #[test]
    fn forms_and_queries_are_encoded_as_browsers_do() {
        let f = |p: &[(&str, &str)]| p.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>();
        assert_eq!(form_encoded(&f(&[("email", "a+b@c.d"), ("note", "x y&z")])), "email=a%2Bb%40c.d&note=x+y%26z");
        assert_eq!(with_query("http://h/s", &f(&[("q", "a b"), ("n", "1")])), "http://h/s?q=a%20b&n=1");
        assert_eq!(with_query("http://h/s?x=1#top", &f(&[("q", "é")])), "http://h/s?x=1&q=%C3%A9#top");
        assert_eq!(with_query("http://h/s", &[]), "http://h/s");
        let c = config(
            "POST",
            "http://h/",
            &[],
            Some(&Body::Form(f(&[("a", "1")]))),
            Duration::from_secs(1),
            How { follow: false, insecure: false },
        );
        assert!(
            c.contains("header = \"Content-Type: application/x-www-form-urlencoded\"\ndata-binary = \"a=1\"\n"),
            "{c}"
        );
        let c = config(
            "POST",
            "http://h/",
            &[],
            Some(&Body::Multipart(f(&[("note", "@not a file?"), ("pic", "@/t/a.png;type=image/png")]))),
            Duration::from_secs(1),
            How { follow: false, insecure: false },
        );
        assert!(c.contains("form = \"note=@not a file?\"\nform = \"pic=@/t/a.png;type=image/png\"\n"), "{c}");
        let c = config(
            "POST",
            "http://h/",
            &[],
            Some(&Body::Multipart(f(&[("note", "plain")]))),
            Duration::from_secs(1),
            How { follow: false, insecure: false },
        );
        assert!(c.contains("form-string = \"note=plain\"\n"), "{c}");
    }

    #[test]
    fn the_jar_sends_back_what_a_browser_would() {
        let now = 1_700_000_000;
        let mut jar = Jar::default();
        jar.store("sid=abc; Path=/; HttpOnly", "http://shop.test/login", now);
        jar.store("theme=dark", "http://shop.test/account/settings", now);
        jar.store("wide=1; Domain=.shop.test", "http://www.shop.test/", now);
        jar.store("evil=1; Domain=other.test", "http://shop.test/", now);
        jar.store("sec=1; Secure", "http://shop.test/", now);
        jar.store("lsec=1; Secure", "http://localhost:8000/", now);
        assert_eq!(jar.header("http://shop.test/", now).as_deref(), Some("sid=abc; wide=1"));
        // The default path is the request's directory, and a longer path goes first.
        assert_eq!(
            jar.header("http://shop.test/account/settings/x", now).as_deref(),
            Some("theme=dark; sid=abc; wide=1")
        );
        assert_eq!(jar.header("http://shop.test/accounts", now).as_deref(), Some("sid=abc; wide=1"));
        // Host-only stays on its host; a domain cookie reaches subdomains.
        assert_eq!(jar.header("http://api.shop.test/", now).as_deref(), Some("wide=1"));
        assert_eq!(jar.header("http://localhost:8000/", now).as_deref(), Some("lsec=1"));
        assert!(jar.header("http://other.test/", now).is_none());
        // Set again: replaced; expired: gone.
        jar.store("sid=new; Path=/", "http://shop.test/", now);
        assert_eq!(jar.get("sid", now).map(|c| c.value.as_str()), Some("new"));
        jar.store("sid=; Max-Age=0; Path=/", "http://shop.test/", now);
        assert!(jar.get("sid", now).is_none());
        jar.store("short=1; Max-Age=5", "http://shop.test/", now);
        assert!(jar.get("short", now + 10).is_none());
        assert!(jar.header("http://shop.test/", now + 10).as_deref() == Some("wide=1"));
    }

    #[test]
    fn a_jar_round_trips_through_navigeras_state_format() {
        let now = 1_700_000_000;
        let mut jar = Jar::default();
        jar.store("sid=abc; Path=/; HttpOnly; SameSite=Strict; Max-Age=60", "https://shop.test/a", now);
        jar.store("wide=1; Domain=shop.test", "https://shop.test/", now);
        let state = jar.state(now);
        assert_eq!(state["cookies"][0]["domain"], "shop.test");
        assert_eq!(state["cookies"][0]["expires"], (now + 60) as f64);
        assert_eq!(state["cookies"][0]["sameSite"], "Strict");
        assert_eq!(state["cookies"][1]["domain"], ".shop.test");
        assert_eq!(state["cookies"][1]["expires"], -1.0);
        let mut back = Jar::default();
        back.load(&state, now);
        assert_eq!(back.state(now), state);
        assert_eq!(back.header("https://shop.test/a/b", now), jar.header("https://shop.test/a/b", now));
        // A browser's own state file (an expired cookie left out).
        let browser = serde_json::json!({"cookies": [
            {"name": "a", "value": "1", "domain": "localhost", "path": "/", "expires": -1},
            {"name": "old", "value": "1", "domain": "localhost", "path": "/", "expires": 5.0}
        ], "origins": [{"origin": "http://localhost", "localStorage": []}]});
        let mut j = Jar::default();
        j.load(&browser, now);
        assert_eq!(j.header("http://localhost:3000/x", now).as_deref(), Some("a=1"));
    }
}
