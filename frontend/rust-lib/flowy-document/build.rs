fn main() {
  #[cfg(feature = "dart")]
  {
    flowy_codegen::protobuf_file::dart_gen(env!("CARGO_PKG_NAME"));
    flowy_codegen::dart_event::r#gen(env!("CARGO_PKG_NAME"));
  }

  // 编译PDFium库
  compile_pdfium();
}

fn compile_pdfium() {
    // 检查是否在目标平台上支持PDFium
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        // 尝试链接系统PDFium库
        if let Ok(pdfium_path) = std::env::var("PDFIUM_PATH") {
            println!("cargo:rustc-link-search=native={}", pdfium_path);
            println!("cargo:rustc-link-lib=pdfium");
        } else {
            // 如果没有找到系统PDFium，尝试编译静态版本
            compile_static_pdfium();
        }
    }
}

fn compile_static_pdfium() {
    // 这里可以实现静态编译PDFium的逻辑
    // 由于PDFium编译比较复杂，这里提供一个框架
    
    // 检查是否有PDFium源码
    let pdfium_src = std::env::var("PDFIUM_SRC").unwrap_or_else(|_| "pdfium".to_string());
    
    if std::path::Path::new(&pdfium_src).exists() {
        println!("cargo:rustc-link-search=native={}/out/Release", pdfium_src);
        println!("cargo:rustc-link-lib=static=pdfium");
        
        // 添加必要的系统库
        if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=pthread");
        } else if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=c++");
        } else if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=gdi32");
        }
    } else {
        // 如果没有PDFium源码，使用系统工具作为备用
        println!("cargo:warning=PDFium source not found, will use system tools as fallback");
    }
}
