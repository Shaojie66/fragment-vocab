import { invoke } from '@tauri-apps/api/core';

const btnKnow = document.getElementById('btnKnow');
const btnUnknown = document.getElementById('btnUnknown');
const btnSkip = document.getElementById('btnSkip');

btnKnow?.addEventListener('click', () => {
    console.log('认识');
    invoke('hide_card_window');
});

btnUnknown?.addEventListener('click', () => {
    console.log('不认识');
    invoke('hide_card_window');
});

btnSkip?.addEventListener('click', () => {
    console.log('跳过');
    invoke('hide_card_window');
});

// 快捷键支持
document.addEventListener('keydown', (e) => {
    if (e.metaKey && e.key === 'k') {
        e.preventDefault();
        btnKnow?.click();
    } else if (e.metaKey && e.key === 'j') {
        e.preventDefault();
        btnUnknown?.click();
    } else if (e.key === 'Escape') {
        e.preventDefault();
        btnSkip?.click();
    }
});
