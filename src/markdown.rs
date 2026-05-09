use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, Options, Plugins};
use once_cell::sync::Lazy;

static HIGHLIGHTER: Lazy<SyntectAdapter> =
    Lazy::new(|| SyntectAdapter::new(Some("base16-ocean.dark")));

pub fn render(md: &str) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.render.unsafe_ = true;

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&*HIGHLIGHTER);

    markdown_to_html_with_plugins(md, &options, &plugins)
}
