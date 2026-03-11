import { invoke } from '@tauri-apps/api/core';
import { triggerScheduler } from './domain/scheduler/triggerScheduler';

console.log('🚀 Fragment Vocab 启动中...');

// 初始化调度器
async function initScheduler() {
    console.log('📅 初始化调度器...');
    
    // 设置触发回调：当满足条件时展示卡片
    triggerScheduler.start(async () => {
        console.log('🎯 触发弹卡');
        
        try {
            // 显示卡片窗口
            await invoke('show_card_window');
            
            // 标记卡片已展示
            triggerScheduler.markCardShown();
            
        } catch (error) {
            console.error('❌ 显示卡片失败:', error);
        }
    });
    
    console.log('✅ 调度器已启动');
}

// 应用启动
window.addEventListener('DOMContentLoaded', () => {
    console.log('📱 应用已加载');
    
    // 启动调度器
    initScheduler();
});

// 监听窗口关闭事件
window.addEventListener('beforeunload', () => {
    console.log('👋 应用关闭中...');
    triggerScheduler.stop();
});
