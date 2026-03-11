const { invoke } = require('@tauri-apps/api/core');

async function testIdle() {
    try {
        const idleSeconds = await invoke('get_idle_seconds');
        console.log(`Current idle time: ${idleSeconds} seconds`);
    } catch (error) {
        console.error('Error:', error);
    }
}

testIdle();
