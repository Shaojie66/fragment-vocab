import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface WordCardData {
    word_id: number;
    card_id: number;
    word: string;
    phonetic?: string;
    part_of_speech?: string;
    meaning_zh: string;
}

let currentCard: WordCardData | null = null;
let autoHideTimer: number | null = null;

const wordEl = document.querySelector('.word') as HTMLElement;
const phoneticEl = document.querySelector('.phonetic') as HTMLElement;
const posEl = document.querySelector('.pos') as HTMLElement;
const meaningEl = document.querySelector('.meaning') as HTMLElement;

const btnKnow = document.getElementById('btnKnow');
const btnUnknown = document.getElementById('btnUnknown');
const btnSkip = document.getElementById('btnSkip');

// 加载并显示卡片
async function loadAndShowCard() {
    try {
        const card = await invoke<WordCardData | null>('get_next_card');
        
        if (!card) {
            console.log('没有可用的卡片');
            await invoke('hide_card_window');
            return;
        }
        
        currentCard = card;
        
        // 更新 UI
        wordEl.textContent = card.word;
        phoneticEl.textContent = card.phonetic || '';
        posEl.textContent = card.part_of_speech || '';
        meaningEl.textContent = card.meaning_zh;
        
        // 设置自动隐藏定时器（10-12 秒）
        const autoHideDelay = 10000 + Math.random() * 2000;
        autoHideTimer = window.setTimeout(() => {
            handleSkip();
        }, autoHideDelay);
        
    } catch (error) {
        console.error('加载卡片失败:', error);
        await invoke('hide_card_window');
    }
}

// 提交复习结果
async function submitReview(result: 'know' | 'dont_know' | 'skip') {
    if (!currentCard) return;
    
    try {
        await invoke('submit_review', {
            cardId: currentCard.card_id,
            result: result,
        });
        
        console.log(`提交结果: ${result}`);
        
        // 清除自动隐藏定时器
        if (autoHideTimer !== null) {
            clearTimeout(autoHideTimer);
            autoHideTimer = null;
        }
        
        // 隐藏窗口
        await invoke('hide_card_window');
        
    } catch (error) {
        console.error('提交复习结果失败:', error);
    }
}

// 处理"认识"
async function handleKnow() {
    console.log('认识');
    await submitReview('know');
}

// 处理"不认识"
async function handleUnknown() {
    console.log('不认识');
    await submitReview('dont_know');
}

// 处理"跳过"
async function handleSkip() {
    console.log('跳过');
    await submitReview('skip');
}

// 按钮事件
btnKnow?.addEventListener('click', handleKnow);
btnUnknown?.addEventListener('click', handleUnknown);
btnSkip?.addEventListener('click', handleSkip);

// 本地快捷键支持（窗口内）
document.addEventListener('keydown', (e) => {
    if (e.metaKey && e.key === 'k') {
        e.preventDefault();
        handleKnow();
    } else if (e.metaKey && e.key === 'j') {
        e.preventDefault();
        handleUnknown();
    } else if (e.key === 'Escape') {
        e.preventDefault();
        handleSkip();
    }
});

// 监听全局快捷键事件
listen('shortcut-know', () => {
    handleKnow();
});

listen('shortcut-dont-know', () => {
    handleUnknown();
});

listen('shortcut-skip', () => {
    handleSkip();
});

// 窗口显示时加载卡片
window.addEventListener('DOMContentLoaded', () => {
    loadAndShowCard();
});
