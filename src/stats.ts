import { invoke } from '@tauri-apps/api/core';

interface TodayStats {
    total_reviews: number;
    know_count: number;
    dont_know_count: number;
    skip_count: number;
    new_words_today: number;
    due_cards_count: number;
    mastered_count: number;
    accuracy: number;
}

async function loadStats() {
    try {
        const stats = await invoke<TodayStats>('get_today_stats');
        
        // 更新 UI
        const statItems = document.querySelectorAll('.stat-item');
        
        if (statItems[0]) {
            const valueEl = statItems[0].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.total_reviews.toString();
        }
        
        if (statItems[1]) {
            const valueEl = statItems[1].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.know_count.toString();
        }
        
        if (statItems[2]) {
            const valueEl = statItems[2].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.dont_know_count.toString();
        }
        
        if (statItems[3]) {
            const valueEl = statItems[3].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.skip_count.toString();
        }
        
        if (statItems[4]) {
            const valueEl = statItems[4].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.new_words_today.toString();
        }
        
        if (statItems[5]) {
            const valueEl = statItems[5].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.due_cards_count.toString();
        }
        
        if (statItems[6]) {
            const valueEl = statItems[6].querySelector('.stat-value');
            if (valueEl) valueEl.textContent = stats.mastered_count.toString();
        }
        
        // 添加正确率显示
        const container = document.querySelector('.container');
        if (container) {
            const accuracyItem = document.createElement('div');
            accuracyItem.className = 'stat-item';
            accuracyItem.innerHTML = `
                <span class="stat-label">今日正确率</span>
                <span class="stat-value">${stats.accuracy.toFixed(1)}%</span>
            `;
            container.appendChild(accuracyItem);
        }
        
    } catch (error) {
        console.error('加载统计数据失败:', error);
    }
}

// 页面加载时获取统计数据
window.addEventListener('DOMContentLoaded', () => {
    loadStats();
    
    // 每 5 秒刷新一次
    setInterval(loadStats, 5000);
});
