// macOS 系统 idle 时间检测
// 使用 Core Graphics API 获取系统空闲时间

#[cfg(target_os = "macos")]
use core_graphics::{
    event::CGEventType,
    event_source::CGEventSourceStateID,
};

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(
        stateID: CGEventSourceStateID,
        eventType: CGEventType,
    ) -> f64;
}

/// 获取系统 idle 秒数
/// 
/// 返回自上次用户输入事件（键盘、鼠标）以来的秒数
#[cfg(target_os = "macos")]
pub fn get_idle_seconds() -> Result<f64, String> {
    let idle_time = unsafe {
        CGEventSourceSecondsSinceLastEventType(
            CGEventSourceStateID::CombinedSessionState,
            CGEventType::Null
        )
    };
    
    Ok(idle_time)
}

/// 非 macOS 平台的占位实现
#[cfg(not(target_os = "macos"))]
pub fn get_idle_seconds() -> Result<f64, String> {
    Err("Idle detection is only supported on macOS".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_idle_seconds() {
        let result = get_idle_seconds();
        assert!(result.is_ok());
        
        let seconds = result.unwrap();
        assert!(seconds >= 0.0);
        println!("Current idle time: {:.2} seconds", seconds);
    }
}
