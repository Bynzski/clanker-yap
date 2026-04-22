/**
 * Voice Transcription UI
 *
 * Tauri API is loaded from src/vendor/tauri.js.
 */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let settings = null;
let transcriptions = [];
let currentError = null;
let modelDownloadInfo = null;
let capturedHotkey = null;
let hotkeyCaptureActive = false;
let selectedPasteMode = "auto";
let historyCollapsed = false;
let audioDevices = [];
let selectedMicName = null;

async function init() {
    showStatus("loading", "Loading", "Initializing");

    try {
        const [settingsData, historyData, statusData] = await Promise.all([
            invoke("get_settings"),
            invoke("get_transcription_history"),
            invoke("get_status"),
        ]);

        settings = settingsData;
        transcriptions = historyData.transcriptions || [];

        updateSettingsUI(settings);
        updateHistoryUI(transcriptions);

        const statusState = (statusData.state || "Idle").toLowerCase();
        showStatus(statusState, getStateLabel(statusData.state), getStateDescription(statusState));

        if (statusData.last_error) {
            showError("transcription", statusData.last_error);
        }

        if (!settings.model_path) {
            showError(
                "model",
                'Model path is empty. <a href="https://huggingface.co/ggerganov/whisper.cpp/tree/main" target="_blank">Download ggml-base.en.bin</a> and configure the local path.',
            );
        }

        await loadModelDownloadInfo();
        await loadAudioDevices();
        updateMicrophoneStatus(settings);
    } catch (err) {
        console.error("Init error:", err);
        showStatus("error", "Unavailable", "Failed to load");
        showError("settings", `Failed to load application state: ${err}`);
    }

    await setupEventListeners();
}

async function loadModelDownloadInfo() {
    try {
        modelDownloadInfo = await invoke("get_default_model_download_info");
        updateModelDownloadUI(modelDownloadInfo);
    } catch (err) {
        console.error("Failed to load model download info:", err);
    }
}

async function setupEventListeners() {
    const events = [
        { name: "recording-started", handler: onRecordingStarted },
        { name: "recording-stopped", handler: onRecordingStopped },
        { name: "transcription-complete", handler: onTranscriptionComplete },
        { name: "transcription-error", handler: onTranscriptionError },
        { name: "hotkey-conflict", handler: onHotkeyConflict },
    ];

    for (const { name, handler } of events) {
        try {
            await listen(name, (event) => handler(event.payload || {}));
        } catch (err) {
            console.error("Failed to listen for", name, err);
        }
    }
}

function onRecordingStarted() {
    showStatus("recording", "Recording", "Listening for speech");
    clearError();
}

function onRecordingStopped(payload) {
    const duration = payload.duration_ms ? `${payload.duration_ms} ms capture` : "Preparing audio";
    showStatus("processing", "Processing", duration);
}

function onTranscriptionComplete(payload) {
    showStatus("idle", "Ready", "Awaiting shortcut");
    clearError();

    const item = {
        id: generateId(),
        text: payload.text,
        duration_ms: payload.duration_ms || 0,
        created_at: new Date().toISOString(),
    };

    transcriptions.unshift(item);
    transcriptions = transcriptions.slice(0, 10);
    updateHistoryUI(transcriptions);
}

function onTranscriptionError(payload) {
    const message = payload.error || "Transcription failed";
    showStatus("error", "Attention required", "Review the latest error");
    showError("transcription", message);
}

function onHotkeyConflict(payload) {
    showStatus("error", "Hotkey conflict", "Shortcut update failed");
    showError("hotkey", `Hotkey conflict: ${payload.hotkey} is already in use.`);
}

function showStatus(state, label, detail) {
    const panel = document.getElementById("status-panel");
    const badge = document.getElementById("status-badge");
    const text = document.getElementById("status-text");
    const chip = document.getElementById("status-chip");
    const mode = document.getElementById("status-mode");

    document.body.classList.remove("status-loading", "status-idle", "status-recording", "status-processing", "status-error");
    document.body.classList.add(`status-${state}`);

    if (panel) panel.dataset.state = state;
    if (badge) badge.dataset.state = state;
    if (text) text.textContent = label;
    if (chip) chip.textContent = detail;
    if (mode) mode.textContent = label;
}

function getStateLabel(state) {
    switch ((state || "").toLowerCase()) {
        case "idle":
            return "Ready";
        case "recording":
            return "Recording";
        case "processing":
            return "Processing";
        case "error":
            return "Attention required";
        default:
            return state || "Loading";
    }
}

function getStateDescription(state) {
    switch (state) {
        case "idle":
            return "Awaiting shortcut";
        case "recording":
            return "Listening for speech";
        case "processing":
            return "Transcribing locally";
        case "error":
            return "Review the latest error";
        default:
            return "Initializing";
    }
}

function updateSettingsUI(nextSettings) {
    const hotkeyEl = document.getElementById("hotkey-value");
    const hotkeyDisplayEl = document.getElementById("hotkey-display");
    const modelNameEl = document.getElementById("model-name");
    const modelPathEl = document.getElementById("model-path");
    const modelInput = document.getElementById("model-input");
    const pasteModeValueEl = document.getElementById("paste-mode-value");
    const pasteModeDescriptionEl = document.getElementById("paste-mode-description");
    const micValueEl = document.getElementById("microphone-value");

    if (hotkeyEl) hotkeyEl.textContent = nextSettings.hotkey || "--";
    if (hotkeyDisplayEl) hotkeyDisplayEl.textContent = formatHotkeyForDisplay(nextSettings.hotkey) || "Press a shortcut with at least one modifier.";
    if (modelNameEl) modelNameEl.textContent = nextSettings.model_name || "--";
    if (modelPathEl) modelPathEl.textContent = nextSettings.model_path || "--";
    if (modelInput) modelInput.value = nextSettings.model_path || "";
    if (pasteModeValueEl) pasteModeValueEl.textContent = formatPasteMode(nextSettings.paste_mode);
    if (pasteModeDescriptionEl) pasteModeDescriptionEl.textContent = getPasteModeDescription(nextSettings.paste_mode);
    selectedPasteMode = nextSettings.paste_mode || "auto";
    updatePasteModeSelector(selectedPasteMode);

    const audioInput = nextSettings.audio_input;
    if (micValueEl) {
        if (!audioInput || audioInput.type === "system_default") {
            micValueEl.textContent = "System Default";
        } else if (audioInput.type === "by_name") {
            micValueEl.textContent = audioInput.value;
        }
    }

    updateMicrophoneStatus(nextSettings);
}

function updateModelDownloadUI(info) {
    const metaEl = document.getElementById("model-download-meta");
    const buttonEl = document.getElementById("download-model-btn");
    if (!metaEl || !buttonEl || !info) return;

    const installedLabel = info.installed ? "Installed at target path." : "Not installed at target path.";
    metaEl.textContent = `${info.model_name} · ${info.size_label} · ${installedLabel}`;
    buttonEl.textContent = info.installed ? "Re-download base.en" : "Download base.en";
    buttonEl.disabled = false;
}

function updateHistoryUI(items) {
    const listEl = document.getElementById("history-list");
    const countEl = document.getElementById("history-count");
    if (!listEl) return;

    if (countEl) countEl.textContent = String(items?.length || 0);

    if (!items || items.length === 0) {
        listEl.innerHTML = `
            <li class="empty-state">
              <span class="empty-title">No transcriptions yet</span>
              <span class="empty-copy">Hold the hotkey and start speaking.</span>
            </li>
        `;
        return;
    }

    listEl.innerHTML = items
        .map((item) => {
            const text = item.text.length > 120 ? `${item.text.slice(0, 120)}…` : item.text;
            const time = relativeTime(item.created_at);
            return `
                <li class="history-item">
                  <span class="history-text">${escapeHtml(text)}</span>
                  <span class="history-meta">${time} · ${item.duration_ms} ms</span>
                </li>
            `;
        })
        .join("");

    updateHistoryCollapseUI();
}

function showError(type, message) {
    currentError = { type, message };
    const banner = document.getElementById("error-banner");
    if (!banner) return;

    let extraContent = "";
    if (type === "model") {
        extraContent = "<p>Place the model locally and point the app at the correct file.</p>";
    } else if (type === "hotkey") {
        extraContent = "<p>Choose a shortcut that is not already claimed by the desktop session.</p>";
    } else if (type === "transcription") {
        extraContent = "<p>The last request did not complete successfully.</p>";
    } else if (type === "settings") {
        extraContent = "<p>Reload the window if the state looks out of sync.</p>";
    }

    banner.innerHTML = `<strong>System notice</strong>${message}${extraContent}`;
    banner.style.display = "block";
}

function clearError() {
    currentError = null;
    const banner = document.getElementById("error-banner");
    if (banner) banner.style.display = "none";
}

async function updateHotkey(newHotkey) {
    try {
        const result = await invoke("update_settings", {
            request: { hotkey: newHotkey },
        });

        if (result.success === false) {
            showError("hotkey", result.message || "Hotkey update failed.");
            showStatus("error", "Hotkey conflict", "Shortcut update failed");
            updateSettingsUI(settings);
            return;
        }

        if (result.requires_restart) {
            showError("settings", "Restart required to apply the hotkey change.");
            showStatus("error", "Restart required", "Hotkey update incomplete");
            return;
        }

        settings.hotkey = newHotkey;
        updateSettingsUI(settings);
        resetHotkeyCaptureState();
        clearError();
        showStatus("idle", "Ready", "Awaiting shortcut");
    } catch (err) {
        showError("settings", `Failed to update hotkey: ${err}`);
        showStatus("error", "Settings error", "Unable to save hotkey");
    }
}

async function updateModelPath(newPath) {
    try {
        const result = await invoke("update_settings", {
            request: { model_path: newPath },
        });

        if (result.success === false) {
            showError("model", result.message || "Model path update failed.");
            showStatus("error", "Model error", "Unable to save path");
            updateSettingsUI(settings);
            return;
        }

        if (result.requires_restart) {
            showError("settings", "Restart required to apply the model change.");
            showStatus("error", "Restart required", "Model update incomplete");
            return;
        }

        settings.model_path = newPath;
        updateSettingsUI(settings);
        clearError();
        showStatus("idle", "Ready", "Awaiting shortcut");
    } catch (err) {
        showError("model", `Failed to update model path: ${err}`);
        showStatus("error", "Model error", "Unable to save path");
    }
}

async function updatePasteMode(newMode) {
    try {
        const result = await invoke("update_settings", {
            request: { paste_mode: newMode },
        });

        if (result.success === false) {
            showError("settings", result.message || "Paste mode update failed.");
            showStatus("error", "Settings error", "Unable to save paste mode");
            updateSettingsUI(settings);
            return;
        }

        settings.paste_mode = newMode;
        selectedPasteMode = newMode;
        updateSettingsUI(settings);
        clearError();
        showStatus("idle", "Ready", "Awaiting shortcut");
    } catch (err) {
        showError("settings", `Failed to update paste mode: ${err}`);
        showStatus("error", "Settings error", "Unable to save paste mode");
    }
}

function selectPasteMode(value) {
    selectedPasteMode = value;
    updatePasteModeSelector(value);
}

function updatePasteModeSelector(value) {
    document.querySelectorAll(".segment-button").forEach((button) => {
        const isActive = button.dataset.value === value;
        button.classList.toggle("active", isActive);
        button.setAttribute("aria-checked", isActive ? "true" : "false");
    });
}

async function confirmModelDownload() {
    if (!modelDownloadInfo) {
        await loadModelDownloadInfo();
    }

    if (!modelDownloadInfo) {
        showError("model", "Model download metadata is unavailable.");
        return;
    }

    const confirmed = window.confirm(
        [
            `Download ${modelDownloadInfo.model_name}?`,
            "",
            `Size: ${modelDownloadInfo.size_label}`,
            `Destination: ${modelDownloadInfo.destination_path}`,
            "",
            "This will download the model into the app data directory and set it as the active model path.",
        ].join("\n"),
    );

    if (!confirmed) {
        return;
    }

    const buttonEl = document.getElementById("download-model-btn");
    if (buttonEl) {
        buttonEl.disabled = true;
        buttonEl.textContent = "Downloading...";
    }

    showStatus("processing", "Downloading model", modelDownloadInfo.size_label);
    clearError();

    try {
        const result = await invoke("download_default_model");
        settings.model_name = result.model_name;
        settings.model_path = result.model_path;
        updateSettingsUI(settings);
        showStatus("idle", "Ready", "Model installed locally");
        await loadModelDownloadInfo();
    } catch (err) {
        showStatus("error", "Download failed", "Model installation did not complete");
        showError(
            "model",
            `Failed to download the model automatically: ${err}. You can still download it manually from <a href="${modelDownloadInfo.source_url}" target="_blank">the upstream file</a>.`,
        );
        if (buttonEl) {
            buttonEl.disabled = false;
            buttonEl.textContent = modelDownloadInfo.installed ? "Re-download base.en" : "Download base.en";
        }
    }
}

function relativeTime(isoString) {
    try {
        const diff = (Date.now() - new Date(isoString).getTime()) / 1000;
        if (diff < 60) return `${Math.max(1, Math.floor(diff))} sec ago`;
        if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
        if (diff < 86400) return `${Math.floor(diff / 3600)} hr ago`;
        return `${Math.floor(diff / 86400)} day ago${diff >= 172800 ? "s" : ""}`;
    } catch {
        return "unknown";
    }
}

function beginHotkeyCapture() {
    showEdit("hotkey");
    hotkeyCaptureActive = true;
    capturedHotkey = null;
    updateHotkeyCaptureUI({
        label: "Press desired shortcut",
        value: "Waiting for input",
        hint: "Use at least one modifier key plus a non-modifier key.",
        valid: false,
        capturing: true,
    });

    const captureEl = document.getElementById("hotkey-capture");
    if (captureEl) {
        captureEl.focus();
    }
}

function handleHotkeyCapture(event) {
    if (!hotkeyCaptureActive) {
        return;
    }

    event.preventDefault();

    if (event.key === "Escape") {
        hideEdit("hotkey");
        return;
    }

    const parsed = parseHotkeyEvent(event);
    updateHotkeyCaptureUI(parsed);
}

function parseHotkeyEvent(event) {
    const modifiers = [];
    const displayModifiers = [];

    if (event.ctrlKey || event.metaKey) {
        modifiers.push("CmdOrCtrl");
        displayModifiers.push(getCommandLabel());
    }
    if (event.shiftKey) {
        modifiers.push("Shift");
        displayModifiers.push("Shift");
    }
    if (event.altKey) {
        modifiers.push("Alt");
        displayModifiers.push("Alt");
    }

    const keyInfo = normalizeHotkeyKey(event.key);
    if (!keyInfo) {
        return {
            label: "Shortcut rejected",
            value: formatPressedModifiers(displayModifiers),
            hint: "Use at least one modifier and one supported key.",
            valid: false,
            capturing: true,
        };
    }

    if (modifiers.length === 0) {
        return {
            label: "Modifier required",
            value: keyInfo.display,
            hint: "Add Ctrl, Cmd, Shift, or Alt to create a global shortcut.",
            valid: false,
            capturing: true,
        };
    }

    const storageValue = [...modifiers, keyInfo.storage].join("+");
    const displayValue = [...displayModifiers, keyInfo.display].join(" + ");

    capturedHotkey = storageValue;

    return {
        label: "Shortcut captured",
        value: displayValue,
        hint: storageValue,
        valid: true,
        capturing: true,
    };
}

function normalizeHotkeyKey(rawKey) {
    if (!rawKey) {
        return null;
    }

    const key = rawKey.length === 1 ? rawKey.toUpperCase() : rawKey;
    const lower = key.toLowerCase();

    if (["control", "shift", "alt", "meta", "os"].includes(lower)) {
        return null;
    }

    if (/^[A-Z]$/.test(key) || /^[0-9]$/.test(key)) {
        return { storage: key, display: key };
    }

    if (/^F([1-9]|1[0-2])$/.test(key.toUpperCase())) {
        return { storage: key.toUpperCase(), display: key.toUpperCase() };
    }

    const map = {
        arrowup: { storage: "Up", display: "Up Arrow" },
        arrowdown: { storage: "Down", display: "Down Arrow" },
        arrowleft: { storage: "Left", display: "Left Arrow" },
        arrowright: { storage: "Right", display: "Right Arrow" },
        enter: { storage: "Enter", display: "Enter" },
        tab: { storage: "Tab", display: "Tab" },
        escape: { storage: "Escape", display: "Escape" },
        esc: { storage: "Escape", display: "Escape" },
        " ": { storage: "Space", display: "Space" },
        spacebar: { storage: "Space", display: "Space" },
        backspace: { storage: "Backspace", display: "Backspace" },
        delete: { storage: "Delete", display: "Delete" },
        home: { storage: "Home", display: "Home" },
        end: { storage: "End", display: "End" },
        pageup: { storage: "PageUp", display: "Page Up" },
        pagedown: { storage: "PageDown", display: "Page Down" },
        insert: { storage: "Insert", display: "Insert" },
    };

    return map[lower] || null;
}

function updateHotkeyCaptureUI({ label, value, hint, valid, capturing }) {
    const captureEl = document.getElementById("hotkey-capture");
    const labelEl = document.getElementById("hotkey-capture-label");
    const valueEl = document.getElementById("hotkey-capture-value");
    const hintEl = document.getElementById("hotkey-capture-hint");
    const saveButtonEl = document.getElementById("save-hotkey-btn");

    if (labelEl) labelEl.textContent = label;
    if (valueEl) valueEl.textContent = value;
    if (hintEl) hintEl.textContent = hint;
    if (saveButtonEl) saveButtonEl.disabled = !valid;
    if (captureEl) {
        captureEl.classList.toggle("capturing", Boolean(capturing));
        captureEl.classList.toggle("invalid", capturing && !valid);
    }
}

function formatPressedModifiers(modifiers) {
    return modifiers.length > 0 ? modifiers.join(" + ") : "Waiting for input";
}

function formatHotkeyForDisplay(value) {
    if (!value) {
        return "";
    }

    return value
        .split("+")
        .map((part) => {
            if (part === "CmdOrCtrl") return getCommandLabel();
            if (part === "Alt") return "Alt";
            if (part === "Shift") return "Shift";
            if (part === "PageUp") return "Page Up";
            if (part === "PageDown") return "Page Down";
            if (part === "Left") return "Left Arrow";
            if (part === "Right") return "Right Arrow";
            if (part === "Up") return "Up Arrow";
            if (part === "Down") return "Down Arrow";
            return part;
        })
        .join(" + ");
}

function formatPasteMode(value) {
    switch (value) {
        case "terminal":
            return "Terminal";
        case "standard":
            return "Standard";
        case "auto":
        default:
            return "Auto";
    }
}

function getPasteModeDescription(value) {
    switch (value) {
        case "terminal":
            return "Uses Linux terminal paste shortcuts such as Ctrl + Shift + V.";
        case "standard":
            return "Uses the normal application paste shortcut.";
        case "auto":
        default:
            return "Uses standard paste behavior by default.";
    }
}

function getCommandLabel() {
    return navigator.platform.toLowerCase().includes("mac") ? "Cmd" : "Ctrl";
}

function clearCapturedHotkey() {
    capturedHotkey = null;
    updateHotkeyCaptureUI({
        label: "Press desired shortcut",
        value: "Waiting for input",
        hint: "Use at least one modifier key plus a non-modifier key.",
        valid: false,
        capturing: hotkeyCaptureActive,
    });
}

function resetHotkeyCaptureState() {
    hotkeyCaptureActive = false;
    clearCapturedHotkey();
    const captureEl = document.getElementById("hotkey-capture");
    if (captureEl) {
        captureEl.classList.remove("capturing", "invalid");
    }
}

function generateId() {
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (char) => {
        const random = Math.random() * 16 | 0;
        const value = char === "x" ? random : (random & 0x3) | 0x8;
        return value.toString(16);
    });
}

function escapeHtml(value) {
    const div = document.createElement("div");
    div.textContent = value;
    return div.innerHTML;
}

function showEdit(field) {
    const editEl = document.getElementById(`edit-${field}`);
    const cardEl = document.getElementById(`${field}-card`);

    if (editEl) editEl.classList.add("active");
    if (cardEl) cardEl.classList.add("editing");

    if (field === "hotkey") {
        const captureEl = document.getElementById("hotkey-capture");
        if (captureEl) {
            captureEl.focus();
        }
    } else if (field === "microphone") {
        loadAudioDevices().then(() => populateMicrophoneSelect());
    } else {
        const input = editEl?.querySelector("input");
        if (input) {
            input.focus();
            input.select();
        }
    }
}

function hideEdit(field) {
    const editEl = document.getElementById(`edit-${field}`);
    const cardEl = document.getElementById(`${field}-card`);

    if (editEl) editEl.classList.remove("active");
    if (cardEl) cardEl.classList.remove("editing");
    if (field === "hotkey") {
        hotkeyCaptureActive = false;
        resetHotkeyCaptureState();
    }
}

function submitHotkey() {
    if (!capturedHotkey) return;
    const value = capturedHotkey;

    hideEdit("hotkey");
    updateHotkey(value);
}

function submitModelPath() {
    const input = document.getElementById("model-input");
    const value = input?.value.trim();
    if (!value) return;

    hideEdit("model");
    updateModelPath(value);
}

function submitPasteMode() {
    if (!selectedPasteMode) return;

    hideEdit("paste");
    updatePasteMode(selectedPasteMode);
}

function toggleHistory() {
    historyCollapsed = !historyCollapsed;
    updateHistoryCollapseUI();
}

function updateHistoryCollapseUI() {
    const listEl = document.getElementById("history-list");
    const toggleEl = document.getElementById("history-toggle");
    if (!listEl || !toggleEl) return;

    listEl.dataset.collapsed = historyCollapsed ? "true" : "false";
    toggleEl.textContent = historyCollapsed ? "Expand" : "Collapse";
}

async function loadAudioDevices() {
    try {
        audioDevices = await invoke("list_audio_inputs");
    } catch (err) {
        console.error("Failed to list audio devices:", err);
        audioDevices = [];
    }
}

function populateMicrophoneSelect() {
    const selectEl = document.getElementById("microphone-select");
    if (!selectEl) return;

    const currentSelection = settings?.audio_input;
    const currentName =
        currentSelection && currentSelection.type === "by_name"
            ? currentSelection.value
            : null;

    let html = '<option value="__system_default__">System Default</option>';

    for (const device of audioDevices) {
        const displayName = device.name + device.name_suffix;
        const defaultTag = device.is_default ? " (Default)" : "";
        const stateTag = device.state === "Unavailable"
            ? " — Currently unavailable"
            : device.state === "FormatUnsupported"
                ? " — Format unsupported"
                : "";
        const label = escapeHtml(displayName + defaultTag + stateTag);
        const selected = device.name === currentName ? " selected" : "";
        html += `<option value="${escapeHtml(device.name)}"${selected}>${label}</option>`;
    }

    if (currentName && !audioDevices.some((d) => d.name === currentName)) {
        html += `<option value="${escapeHtml(currentName)}" selected>${escapeHtml(currentName)} — Currently unavailable</option>`;
    }

    selectEl.innerHTML = html;

    if (!currentName) {
        selectEl.value = "__system_default__";
    }
}

function updateMicrophoneStatus(nextSettings) {
    const statusEl = document.getElementById("microphone-status");
    if (!statusEl) return;

    const audioInput = nextSettings.audio_input;
    if (!audioInput || audioInput.type === "system_default") {
        statusEl.textContent = "Using the operating system's default input device.";
        return;
    }

    const name = audioInput.value;
    const device = audioDevices.find((d) => d.name === name);

    if (!device) {
        statusEl.textContent = "Selected device is currently unavailable. Will fall back to system default.";
        return;
    }

    if (device.state === "Unavailable") {
        statusEl.textContent = "Device detected but currently unavailable.";
    } else if (device.state === "FormatUnsupported") {
        statusEl.textContent = "Device has no supported input format.";
    } else {
        statusEl.textContent = "Available.";
    }
}

async function submitMicrophone() {
    const selectEl = document.getElementById("microphone-select");
    if (!selectEl) return;

    const value = selectEl.value;
    hideEdit("microphone");

    const audioInput =
        value === "__system_default__"
            ? { type: "system_default" }
            : { type: "by_name", value: value };

    try {
        const result = await invoke("update_settings", {
            request: { audio_input: audioInput },
        });

        if (result.success === false) {
            showError("settings", result.message || "Microphone update failed.");
            return;
        }

        settings.audio_input = audioInput;
        updateSettingsUI(settings);
        clearError();
    } catch (err) {
        showError("settings", `Failed to update microphone: ${err}`);
    }
}

window.showEdit = showEdit;
window.hideEdit = hideEdit;
window.submitHotkey = submitHotkey;
window.submitModelPath = submitModelPath;
window.submitPasteMode = submitPasteMode;
window.submitMicrophone = submitMicrophone;
window.confirmModelDownload = confirmModelDownload;
window.beginHotkeyCapture = beginHotkeyCapture;
window.clearCapturedHotkey = clearCapturedHotkey;
window.selectPasteMode = selectPasteMode;
window.toggleHistory = toggleHistory;

document.addEventListener("DOMContentLoaded", init);
document.addEventListener("keydown", handleHotkeyCapture, true);
