use mdbook_katex::{cfg::KatexConfig, preprocess::KATEX_HEADER, render::process_chapter_prerender};

fn main() {
    let cfg = KatexConfig::default();
    let extra_opts = cfg.build_extra_opts();
    let (inline_opts, display_opts) = cfg.build_opts("");
    let stylesheet_header = KATEX_HEADER.to_owned();
    
    let raw_content = r"Same pattern shows $\overline{INT\_EMPTY_{CFG}}$ is r.e., hence $INT\_EMPTY_{CFG}$ is co-r.e.";
    
    let rendered = process_chapter_prerender(
        raw_content,
        inline_opts,
        display_opts,
        &stylesheet_header,
        &extra_opts,
    );
    
    println!("INPUT: {}", raw_content);
    println!("\nOUTPUT LENGTH: {}", rendered.len());
    println!("\nOUTPUT:");
    println!("{}", rendered);
}
