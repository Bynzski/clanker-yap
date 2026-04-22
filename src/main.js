/**
 * Voice Transcription — Main JavaScript
 * 
 * Tauri API v2 (jsdelivr CDN):
 * https://cdn.jsdelivr.net/npm/@tauri-apps/api@2/dist/tauri.min.js
 */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ── State ───────────────────────────────────────────────────────────────────

let settings = null;
let transcriptions = [];
let currentError = null;

// ── Init ────────────────────────────────────────────────────────────────────

async function init() {
    showStatus('loading', 'Loading…');

    try {
        // Load settings + history in parallel
        const [settingsData, historyData, statusData] = await Promise.all([
            invoke('get_settings'),
            invoke('get_transcription_history'),
            invoke('get_status'),
        ]);

        settings = settingsData;
        transcriptions = historyData.transcriptions || [];
        updateSettingsUI(settings);
        updateHistoryUI(transcriptions);
        showStatus(statusData.state.toLowerCase(), getStateLabel(statusData.state));

        // Check for model not found on load
        if (!settings.model_path || !fileExists(settings.model_path)) {
            showError('model', 'Model not found. <a href="https://huggingface.co/ggerganov/whisper.cpp/tree/main" target="_blank">Download ggml-base.en.bin</a> and place it at the path shown above.');
        }
    } catch (err) {
        console.error('Init error:', err);
        showStatus('error', 'Failed to load');
    }

    // Set up event listeners
    await setupEventListeners();
}

// ── Event Listeners ─────────────────────────────────────────────────────────

async function setupEventListeners() {
    const events = [
        { name: 'recording-started', handler: onRecordingStarted },
        { name: 'recording-stopped', handler: onRecordingStopped },
        { name: 'transcription-complete', handler: onTranscriptionComplete },
        { name: 'transcription-error', handler: onTranscriptionError },
        { name: 'hotkey-conflict', handler: onHotkeyConflict },
    ];

    for (const { name, handler } of events) {
        try {
            await listen(name, (event) => handler(event.payload || {}));
            console.log('Listening for:', name);
        } catch (err) {
            console.error('Failed to listen for', name, err);
        }
    }
}

// ── Event Handlers ─────────────────────────────────────────────────────────

function onRecordingStarted() {
    showStatus('recording', 'Recording…');
    clearError();
}

function onRecordingStopped(payload) {
    showStatus('processing', `Processing (${payload.duration_ms}ms)…`);
}

function onTranscriptionComplete(payload) {
    showStatus('idle', 'Idle');
    clearError();

    // Add to history
    const item = {
        id: generateId(),
        text: payload.text,
        duration_ms: payload.duration_ms || 0,
        created_at: new Date().toISOString(),
    };
    transcriptions.unshift(item);

    // Keep max 10
    if (transcriptions.length > 10) {
        transcriptions = transcriptions.slice(0, 10);
    }

    updateHistoryUI(transcriptions);
}

function onTranscriptionError(payload) {
    showStatus('error', payload.error || 'Transcription failed');
    clearError();
    showError('transcription', payload.error || 'Transcription failed');
}

function onHotkeyConflict(payload) {
    showError('hotkey', `Hotkey conflict: ${payload.hotkey} is already in use. Please choose a different hotkey.`);
}

// ── UI Updates ──────────────────────────────────────────────────────────────

function showStatus(state, label) {
    const statusEl = document.getElementById('status');
    const statusTextEl = document.getElementById('status-text');
    if (!statusEl || !statusTextEl) return;

    // Update indicator dot color
    statusEl.className = `status-indicator status-${state}`;

    // Update label
    statusTextEl.textContent = label;
}

function getStateLabel(state) {
    switch (state.toLowerCase()) {
        case 'idle': return 'Idle';
        case 'recording': return 'Recording…';
        case 'processing': return 'Processing…';
        case 'error': return 'Error';
        default: return state;
    }
}

function updateSettingsUI(s) {
    const hotkeyEl = document.getElementById('hotkey-value');
    const modelNameEl = document.getElementById('model-name');
    const modelPathEl = document.getElementById('model-path');

    if (hotkeyEl) hotkeyEl.textContent = s.hotkey;
    if (modelNameEl) modelNameEl.textContent = s.model_name;
    if (modelPathEl) modelPathEl.textContent = s.model_path;
}

function updateHistoryUI(items) {
    const listEl = document.getElementById('history-list');
    if (!listEl) return;

    if (!items || items.length === 0) {
        listEl.innerHTML = '<p class="empty-state">No transcriptions yet. Hold the hotkey to start.</p>';
        return;
    }

    listEl.innerHTML = items.map(item => {
        const text = item.text.length > 80 ? item.text.slice(0, 80) + '…' : item.text;
        const time = relativeTime(item.created_at);
        return `<li class="history-item">
            <span class="history-text">"${escapeHtml(text)}"</span>
            <span class="history-meta">${time} · ${item.duration_ms}ms</span>
        </li>`;
    }).join('');
}

// ── Error Banner ─────────────────────────────────────────────────────────────

function showError(type, message) {
    currentError = { type, message };
    const banner = document.getElementById('error-banner');
    if (!banner) return;

    let extraContent = '';
    if (type === 'model') {
        // HTML in message is expected
    } else if (type === 'mic') {
        extraContent = '<p>Check microphone permissions in system settings.</p>';
    } else if (type === 'hotkey') {
        extraContent = '<p>Try a different hotkey combination.</p>';
    }

    banner.innerHTML = `<strong>Error:</strong> ${message}${extraContent}`;
    banner.style.display = 'block';
}

function clearError() {
    currentError = null;
    const banner = document.getElementById('error-banner');
    if (banner) banner.style.display = 'none';
}

// ── Settings Updates ─────────────────────────────────────────────────────────

async function updateHotkey(newHotkey) {
    try {
        const result = await invoke('update_settings', {
            request: { hotkey: newHotkey }
        });

        if (result.requires_restart) {
            showError('restart', 'Restart required to apply hotkey change.');
        } else {
            settings.hotkey = newHotkey;
            updateSettingsUI(settings);
            clearError();
            showStatus('idle', 'Idle');
        }
    } catch (err) {
        showError('settings', `Failed to update hotkey: ${err}`);
    }
}

async function updateModelPath(newPath) {
    try {
        const result = await invoke('update_settings', {
            request: { model_path: newPath }
        });

        settings.model_path = newPath;
        updateSettingsUI(settings);
        clearError();
        showStatus('idle', 'Idle');
    } catch (err) {
        showError('model', `Failed to update model path: ${err}`);
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function relativeTime(isoString) {
    try {
        const diff = (Date.now() - new Date(isoString).getTime()) / 1000;
        if (diff < 60) return `${Math.floor(diff)}s ago`;
        if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
        if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
        return `${Math.floor(diff / 86400)}d ago`;
    } catch {
        return 'unknown';
    }
}

function generateId() {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
        const r = Math.random() * 16 | 0;
        const v = c === 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

function fileExists(path) {
    // We can't synchronously check file existence from JS,
    // but we show the path and trust the Rust error if missing.
    return path && path.length > 0;
}

// ── Inline Edit Helpers ────────────────────────────────────────────────────

function showEdit(field) {
    const valueEl = document.getElementById(field + '-value');
    const btnEl = valueEl ? valueEl.nextElementSibling : null;
    if (valueEl) valueEl.style.display = 'none';
    if (btnEl && btnEl.tagName === 'BUTTON') btnEl.style.display = 'none';

    const editEl = document.getElementById('edit-' + field);
    if (editEl) {
        editEl.classList.add('active');
        const input = editEl.querySelector('input');
        if (input) { input.focus(); input.select(); }
    }
}

function hideEdit(field) {
    const editEl = document.getElementById('edit-' + field);
    if (editEl) editEl.classList.remove('active');

    const valueEl = document.getElementById(field + '-value');
    const btnEl = valueEl ? valueEl.nextElementSibling : null;
    if (valueEl) valueEl.style.display = '';
    if (btnEl && btnEl.tagName === 'BUTTON') btnEl.style.display = '';
}

function submitHotkey() {
    const input = document.getElementById('hotkey-input');
    if (!input || !input.value.trim()) return;
    hideEdit('hotkey');
    updateHotkey(input.value.trim());
}

function submitModelPath() {
    const input = document.getElementById('model-input');
    if (!input || !input.value.trim()) return;
    hideEdit('model');
    updateModelPath(input.value.trim());
}

// Expose for inline onclick handlers
window.showEdit = showEdit;
window.hideEdit = hideEdit;
window.submitHotkey = submitHotkey;
window.submitModelPath = submitModelPath;
window.updateHotkey = updateHotkey;
window.updateModelPath = updateModelPath;

// ── Start ────────────────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', init);