//! macOS 文件打开处理模块
//!
//! 处理 macOS 右键"打开方式"和双击打开文件的事件

#![allow(deprecated)]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;

/// 全局存储通过 Apple Event 接收的文件路径
/// 由 NSApplicationDelegate 回调填充，由主应用循环消费
#[cfg(target_os = "macos")]
use once_cell::sync::Lazy;
#[cfg(target_os = "macos")]
use std::sync::Mutex;

#[cfg(target_os = "macos")]
static PENDING_FILES: Lazy<Mutex<Vec<PathBuf>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// 设置 macOS 文件打开处理程序
///
/// 创建实现 `application:openFile:`、`application:openFiles:` 和
/// `applicationShouldHandleReopen:hasVisibleWindows:` 的委托类
/// 以接收通过"打开方式"菜单或双击关联文件类型打开应用时的文件路径
#[cfg(target_os = "macos")]
pub fn setup_file_open_handler() {
    use cocoa::appkit::NSApplication;
    use cocoa::base::{id, nil};
    use objc::declare::ClassDecl;
    use objc::runtime::{Object, Sel};
    #[allow(unused_imports)]
    use objc::runtime::Class;
    use objc::{class, msg_send, sel, sel_impl};

    /// Objective-C 回调 `application:openFile:` 委托方法
    ///
    /// # Safety
    /// 当系统向应用发送打开单个文件事件时，由 Objective-C 运行时调用
    extern "C" fn open_file_callback(
        _this: &Object,
        _sel: Sel,
        _app: id,
        filename: id,
    ) -> objc::runtime::BOOL {
        unsafe {
            if filename != nil {
                // 从 NSString 获取 UTF-8 字符串
                let utf8_string: *const std::os::raw::c_char = msg_send![filename, UTF8String];
                if !utf8_string.is_null() {
                    let c_str = std::ffi::CStr::from_ptr(utf8_string);
                    if let Ok(path_str) = c_str.to_str() {
                        let path = PathBuf::from(path_str);
                        tracing::info!("接收到单个文件打开事件: {:?}", path);
                        // 将路径存储在全局待处理文件列表中
                        if let Ok(mut pending) = PENDING_FILES.lock() {
                            pending.push(path);
                        }
                    }
                }
            }
            objc::runtime::YES
        }
    }

    /// Objective-C 回调 `application:openFiles:` 委托方法
    ///
    /// # Safety
    /// 当系统向应用发送打开多个文件事件时，由 Objective-C 运行时调用
    extern "C" fn open_files_callback(
        _this: &Object,
        _sel: Sel,
        _app: id,
        filenames: id,
    ) {
        unsafe {
            if filenames != nil {
                let count: usize = msg_send![filenames, count];
                tracing::info!("接收到 {} 个文件的打开事件", count);

                if let Ok(mut pending) = PENDING_FILES.lock() {
                    for i in 0..count {
                        let filename: id = msg_send![filenames, objectAtIndex: i];
                        if filename != nil {
                            let utf8_string: *const std::os::raw::c_char = msg_send![filename, UTF8String];
                            if !utf8_string.is_null() {
                                let c_str = std::ffi::CStr::from_ptr(utf8_string);
                                if let Ok(path_str) = c_str.to_str() {
                                    let path = PathBuf::from(path_str);
                                    tracing::info!("处理文件[{}]: {:?}", i, path);
                                    pending.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Objective-C 回调 `applicationShouldHandleReopen:hasVisibleWindows:` 委托方法
    ///
    /// # Safety
    /// 当用户点击 Dock 图标或使用 Command+Tab 切换到应用时调用
    extern "C" fn should_handle_reopen_callback(
        _this: &Object,
        _sel: Sel,
        _app: id,
        _has_visible_windows: objc::runtime::BOOL,
    ) -> objc::runtime::BOOL {
        tracing::info!("接收到应用重新激活事件 (Dock 点击或 Command+Tab)");
        // 返回 YES 表示应用应该处理重新打开事件
        objc::runtime::YES
    }

    unsafe {
        // 创建扩展 NSObject 的自定义应用委托类
        let superclass = class!(NSObject);
        let mut decl = match ClassDecl::new("OASAppDelegate", superclass) {
            Some(d) => d,
            None => {
                tracing::warn!("创建 OASAppDelegate 类失败，类可能已存在");
                return;
            }
        };

        // 添加 application:openFile: 选择器方法
        // 签名: (self, _cmd, application, filename) -> BOOL
        decl.add_method(
            sel!(application:openFile:),
            open_file_callback
                as extern "C" fn(&Object, Sel, id, id) -> objc::runtime::BOOL,
        );

        // 添加 application:openFiles: 选择器方法（处理多个文件）
        // 签名: (self, _cmd, application, filenames)
        decl.add_method(
            sel!(application:openFiles:),
            open_files_callback as extern "C" fn(&Object, Sel, id, id),
        );

        // 添加 applicationShouldHandleReopen:hasVisibleWindows: 选择器方法
        // 签名: (self, _cmd, application, hasVisibleWindows) -> BOOL
        decl.add_method(
            sel!(applicationShouldHandleReopen:hasVisibleWindows:),
            should_handle_reopen_callback
                as extern "C" fn(&Object, Sel, id, objc::runtime::BOOL) -> objc::runtime::BOOL,
        );

        // 注册类并创建实例
        let delegate_class = decl.register();
        let delegate: id = msg_send![delegate_class, new];

        // 在共享 NSApplication 实例上设置委托
        let app = NSApplication::sharedApplication(nil);
        let _: () = msg_send![app, setDelegate: delegate];

        tracing::info!("macOS 文件打开处理程序注册成功 (支持单文件和多文件打开)");
    }
}

/// 非 macOS 平台的空操作
#[cfg(not(target_os = "macos"))]
pub fn setup_file_open_handler() {}

/// 检索并清除通过 Apple Event 接收的所有待处理文件路径
///
/// 如果自上次调用以来接收到文件打开事件，则返回文件列表
/// 如果没有接收到事件，则返回空列表
#[cfg(target_os = "macos")]
pub fn get_pending_files() -> Vec<PathBuf> {
    PENDING_FILES.lock().ok().map(|mut pending| std::mem::take(&mut *pending)).unwrap_or_default()
}

/// 非 macOS 平台始终返回空列表
#[cfg(not(target_os = "macos"))]
pub fn get_pending_files() -> Vec<PathBuf> {
    Vec::new()
}

/// 检索并清除通过 Apple Event 接收的任何待处理文件路径（向后兼容）
///
/// 如果自上次调用以来接收到文件打开事件，则返回 `Some(PathBuf)`
/// 如果没有接收到事件，则返回 `None`
#[cfg(target_os = "macos")]
pub fn get_pending_file() -> Option<PathBuf> {
    PENDING_FILES.lock().ok().and_then(|mut pending| pending.drain(..).next())
}

/// 非 macOS 平台始终返回 None
#[cfg(not(target_os = "macos"))]
pub fn get_pending_file() -> Option<PathBuf> {
    None
}
