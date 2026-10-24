mod cli;

use clap::Parser;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::redirect::Policy;
use std::fs;
use std::io::Write;

/// Maximum number of redirects followed with -L/--location.
const MAX_REDIRECTS: usize = 10;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("curl 0.1.0");
        return;
    }

    let url = args.url.as_deref().unwrap_or_else(|| {
        eprintln!("curl: missing URL");
        std::process::exit(1);
    });

    std::process::exit(run(&args, url));
}

fn run(args: &cli::Args, url: &str) -> i32 {
    let method = match effective_method(args.method.as_deref(), args.head, args.data.is_some()) {
        Ok(m) => m,
        Err(e) => return fail(args, &e, 1),
    };

    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return fail(args, &e, 1),
    };

    // GNU curl does not follow redirects by default; with -L we follow
    // manually, which also keeps the hop limit observable.
    let mut current_url = url.to_string();
    let mut current_method = method.to_string();
    let mut redirects = 0;

    let response = loop {
        let request = match build_request(&client, &current_method, &current_url, args) {
            Ok(r) => r,
            Err(e) => return fail(args, &e, 1),
        };

        let response = match request.send() {
            Ok(r) => r,
            Err(e) => return fail(args, &format!("curl: connection failed: {}", e), 1),
        };

        let status = response.status();
        if args.location
            && status.is_redirection()
            && let Some(location) = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
        {
            if redirects >= MAX_REDIRECTS {
                return fail(
                    args,
                    &format!("curl: too many redirects (limit {})", MAX_REDIRECTS),
                    1,
                );
            }
            match resolve_location(&current_url, location) {
                Some(next) => {
                    redirects += 1;
                    current_url = next;
                    current_method = redirect_method(&current_method, status.as_u16());
                    continue;
                }
                None => {
                    return fail(
                        args,
                        &format!("curl: invalid redirect Location '{}'", location),
                        1,
                    );
                }
            }
        }
        break response;
    };

    let status = response.status();

    if args.fail && status.as_u16() >= 400 {
        return fail(
            args,
            &format!("curl: (22) The requested URL returned error: {}", status),
            22,
        );
    }

    if args.include {
        println!("{:?} {}", response.version(), status);
        for (key, value) in response.headers() {
            println!("{}: {}", key, value.to_str().unwrap_or(""));
        }
        println!();
    }

    let body = match response.bytes() {
        Ok(b) => b,
        Err(e) => return fail(args, &format!("curl: read error: {}", e), 1),
    };

    if let Some(output) = &args.output {
        if let Err(e) = fs::write(output, &body) {
            return fail(
                args,
                &format!("curl: cannot write to {}: {}", output.display(), e),
                1,
            );
        }
    } else {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        if let Err(e) = lock.write_all(&body) {
            return fail(args, &format!("curl: write error: {}", e), 1);
        }
        let _ = lock.flush();
    }

    0
}

/// GNU curl method inference: -X wins over everything, -I is HEAD,
/// -d implies POST, otherwise GET.
fn effective_method(
    explicit: Option<&str>,
    head: bool,
    has_data: bool,
) -> Result<&'static str, String> {
    if let Some(m) = explicit {
        return match m.to_uppercase().as_str() {
            "GET" => Ok("GET"),
            "POST" => Ok("POST"),
            "PUT" => Ok("PUT"),
            "DELETE" => Ok("DELETE"),
            "HEAD" => Ok("HEAD"),
            other => Err(format!("curl: unsupported method '{}'", other)),
        };
    }
    if head {
        return Ok("HEAD");
    }
    if has_data {
        return Ok("POST");
    }
    Ok("GET")
}

/// curl switches the method on redirects: 301/302/303 rewrite POST to GET;
/// 307/308 (and every other method) are preserved.
fn redirect_method(current: &str, status: u16) -> String {
    match status {
        301..=303 if current.eq_ignore_ascii_case("POST") => "GET".to_string(),
        _ => current.to_string(),
    }
}

/// Resolve the Location header of a redirect against the current URL.
/// Absolute locations are kept, relative ones are joined to the base.
fn resolve_location(base: &str, location: &str) -> Option<String> {
    reqwest::Url::parse(base)
        .ok()?
        .join(location)
        .ok()
        .map(|u| u.to_string())
}

/// Redirects are never followed by the client itself (matching GNU curl's
/// default); -L implements following manually in run().
fn build_client() -> Result<Client, String> {
    Client::builder()
        .redirect(Policy::none())
        .build()
        .map_err(|e| format!("curl: client build failed: {}", e))
}

fn build_request(
    client: &Client,
    method: &str,
    url: &str,
    args: &cli::Args,
) -> Result<RequestBuilder, String> {
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        other => return Err(format!("curl: unsupported method '{}'", other)),
    };

    if let Some(data) = &args.data {
        // After a POST->GET redirect the body is dropped, matching curl.
        if !method.eq_ignore_ascii_case("GET") {
            request = request.body(data.clone());
        }
    }

    for header in &args.headers {
        if let Some((key, value)) = header.split_once(':') {
            request = request.header(key.trim(), value.trim());
        }
    }

    Ok(request)
}

/// Report an error unless silent mode (-s) suppresses it, and return the
/// process exit code.
fn fail(args: &cli::Args, message: &str, code: i32) -> i32 {
    if !args.silent {
        eprintln!("{}", message);
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_defaults_to_get() {
        assert_eq!(effective_method(None, false, false), Ok("GET"));
    }

    #[test]
    fn data_implies_post() {
        assert_eq!(effective_method(None, false, true), Ok("POST"));
    }

    #[test]
    fn explicit_request_overrides_data() {
        assert_eq!(effective_method(Some("PUT"), false, true), Ok("PUT"));
        assert_eq!(effective_method(Some("get"), false, true), Ok("GET"));
    }

    #[test]
    fn head_flag_implies_head_method() {
        assert_eq!(effective_method(None, true, false), Ok("HEAD"));
        assert_eq!(effective_method(None, true, true), Ok("HEAD"));
    }

    #[test]
    fn explicit_request_overrides_head_flag() {
        assert_eq!(effective_method(Some("GET"), true, false), Ok("GET"));
    }

    #[test]
    fn unsupported_method_is_rejected() {
        assert!(effective_method(Some("BREW"), false, false).is_err());
    }

    #[test]
    fn redirect_method_rewrites_post_to_get_on_301_302_303() {
        for status in [301u16, 302, 303] {
            assert_eq!(redirect_method("POST", status), "GET");
        }
    }

    #[test]
    fn redirect_method_preserves_method_on_307_308() {
        for status in [307u16, 308] {
            assert_eq!(redirect_method("POST", status), "POST");
            assert_eq!(redirect_method("GET", status), "GET");
        }
    }

    #[test]
    fn redirect_method_preserves_non_post_methods() {
        for status in [301u16, 302, 303] {
            assert_eq!(redirect_method("GET", status), "GET");
            assert_eq!(redirect_method("HEAD", status), "HEAD");
        }
    }

    #[test]
    fn location_resolves_absolute_and_relative() {
        let base = "https://example.com/a/b";
        assert_eq!(
            resolve_location(base, "https://other.net/x").as_deref(),
            Some("https://other.net/x")
        );
        assert_eq!(
            resolve_location(base, "/c").as_deref(),
            Some("https://example.com/c")
        );
        assert_eq!(
            resolve_location(base, "d").as_deref(),
            Some("https://example.com/a/d")
        );
    }

    #[test]
    fn location_with_invalid_base_is_none() {
        assert_eq!(resolve_location("not a url", "/c"), None);
    }

    #[test]
    fn client_is_built_without_redirect_following() {
        // The client must build successfully with Policy::none(); following
        // is done manually only when -L is given.
        assert!(build_client().is_ok());
    }
}
