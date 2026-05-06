use minijinja::value::{Kwargs, Value};
use minijinja::{path_loader, AutoEscape, Environment, Error, ErrorKind, Output, State};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::path::Path;

/// Custom formatter that matches Jinja2's HTML escape (does NOT escape `/`).
fn jinja2_html_formatter(
    out: &mut Output,
    _state: &State,
    value: &Value,
) -> Result<(), Error> {
    if value.is_safe() {
        write!(out, "{value}").map_err(Error::from)?;
        return Ok(());
    }
    let auto_escape = match _state.auto_escape() {
        AutoEscape::Html => true,
        AutoEscape::None => false,
        _ => return minijinja::escape_formatter(out, _state, value),
    };
    if !auto_escape {
        write!(out, "{value}").map_err(Error::from)?;
        return Ok(());
    }
    if let Some(s) = value.as_str() {
        write_jinja2_html(out, s).map_err(Error::from)?;
    } else if value.is_undefined() || value.is_none() {
        // emit nothing
    } else {
        let stringified = value.to_string();
        write_jinja2_html(out, &stringified).map_err(Error::from)?;
    }
    Ok(())
}

fn write_jinja2_html(out: &mut Output, s: &str) -> std::fmt::Result {
    let mut last = 0;
    for (i, b) in s.bytes().enumerate() {
        let escape = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&#34;",
            b'\'' => "&#39;",
            _ => continue,
        };
        if last < i {
            out.write_str(&s[last..i])?;
        }
        out.write_str(escape)?;
        last = i + 1;
    }
    if last < s.len() {
        out.write_str(&s[last..])?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestCtx {
    pub url: String,
    pub url_root: String,
    pub base_url: String,
}

fn read_manifest(path: &Path) -> JsonValue {
    let text = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&text).unwrap_or(JsonValue::Null)
}

fn lookup_asset(manifest: &JsonValue, entry: &str, kind: &str) -> String {
    if let Some(chunk) = manifest.get(entry) {
        if kind == "css" {
            if let Some(css_arr) = chunk.get("css").and_then(|v| v.as_array()) {
                if let Some(first) = css_arr.first().and_then(|v| v.as_str()) {
                    return format!("/static/{first}");
                }
            }
        }
        if let Some(file) = chunk.get("file").and_then(|v| v.as_str()) {
            return format!("/static/{file}");
        }
    }
    format!("/static/{entry}")
}

pub fn build_env(templates_dir: &Path, manifest_path: &Path) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(path_loader(templates_dir));
    env.set_formatter(jinja2_html_formatter);

    // Vite manifest:
    // - debug builds re-read on every call so Vite watcher rebuilds show up immediately
    // - release builds load once at startup and reuse the cached value
    #[cfg(debug_assertions)]
    {
        let path = manifest_path.to_path_buf();
        env.add_function(
            "vite_asset",
            move |entry: String, kind: Option<String>| -> Result<String, Error> {
                let kind = kind.unwrap_or_else(|| "file".to_string());
                let manifest = read_manifest(&path);
                Ok(lookup_asset(&manifest, &entry, &kind))
            },
        );
    }
    #[cfg(not(debug_assertions))]
    {
        let manifest = read_manifest(manifest_path);
        env.add_function(
            "vite_asset",
            move |entry: String, kind: Option<String>| -> Result<String, Error> {
                let kind = kind.unwrap_or_else(|| "file".to_string());
                Ok(lookup_asset(&manifest, &entry, &kind))
            },
        );
    }

    env.add_function("url_for", url_for);

    env
}

fn url_for(state: &State, endpoint: String, kwargs: Kwargs) -> Result<String, Error> {
    let take_str = |k: &str| -> Result<Option<String>, Error> {
        let v: Option<Value> = kwargs.get(k).ok();
        match v {
            None => Ok(None),
            Some(val) => {
                if val.is_undefined() || val.is_none() {
                    Ok(None)
                } else {
                    Ok(Some(val.to_string()))
                }
            }
        }
    };

    let external: bool = kwargs.get("_external").unwrap_or(false);

    let path = match endpoint.as_str() {
        "index" => "/".to_string(),
        "blog_index" => "/blog/".to_string(),
        "blog_post" => {
            let slug = take_str("slug")?.unwrap_or_default();
            format!("/posts/{}/", urlencoding::encode(&slug))
        }
        "blog_post_pdf" => {
            let slug = take_str("slug")?.unwrap_or_default();
            format!("/posts/{}/pdf/", urlencoding::encode(&slug))
        }
        "blog_post_md" => {
            let slug = take_str("slug")?.unwrap_or_default();
            format!("/posts/{}/md/", urlencoding::encode(&slug))
        }
        "blog_tag" => {
            let tag = take_str("tag")?.unwrap_or_default();
            format!("/blog/tag/{}/", urlencoding::encode(&tag))
        }
        "blog_year" => {
            let year = take_str("year")?.unwrap_or_default();
            format!("/blog/year/{}/", urlencoding::encode(&year))
        }
        "content_images" => {
            let filename = take_str("filename")?.unwrap_or_default();
            format!("/content/images/{filename}")
        }
        "og_image" => {
            let slug = take_str("slug")?.unwrap_or_default();
            format!("/og/{slug}.svg")
        }
        "search" => "/search/".to_string(),
        "static" => {
            let filename = take_str("filename")?.unwrap_or_default();
            format!("/static/{filename}")
        }
        other => {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("unknown route in url_for: {other}"),
            ));
        }
    };

    // Allow assert_all_used to pass with our optional kwargs.
    kwargs.assert_all_used()?;

    if external {
        let url_root: Option<String> = state
            .lookup("request")
            .and_then(|req| req.get_attr("url_root").ok())
            .and_then(|v| if v.is_undefined() { None } else { Some(v.to_string()) });
        if let Some(mut root) = url_root {
            if root.ends_with('/') {
                root.pop();
            }
            return Ok(format!("{root}{path}"));
        }
    }
    Ok(path)
}
