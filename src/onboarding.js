// Onboarding wizard — step logic and IPC commands added incrementally in Stories 2.2–2.5
import { invoke } from "@tauri-apps/api/core";

console.debug("[ICantSpell] onboarding window loaded");

// ── Step Navigation ──────────────────────────────────────────────────────────

function showStep(stepId) {
  document.querySelectorAll(".step").forEach((el) => el.classList.remove("active"));
  const target = document.getElementById(stepId);
  if (target) {
    target.classList.add("active");
  } else {
    console.warn(`[ICantSpell] onboarding: step '${stepId}' not found`);
  }
}

// ── Welcome Step ─────────────────────────────────────────────────────────────

const btnGetStarted = document.getElementById("btn-get-started");
if (btnGetStarted) {
  btnGetStarted.addEventListener("click", () => {
    showStep("step-accessibility");
  });
}

// ── Accessibility Step ────────────────────────────────────────────────────────

let accessibilityPollInterval = null;

function stopAccessibilityPolling() {
  if (accessibilityPollInterval !== null) {
    clearInterval(accessibilityPollInterval);
    accessibilityPollInterval = null;
  }
}

function setAccessibilityStatus(message, isWarning = false) {
  const statusEl = document.getElementById("accessibility-status");
  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = isWarning ? "status-warning" : "status-info";
  }
}

async function startAccessibilityPolling() {
  stopAccessibilityPolling();
  const btn = document.getElementById("btn-grant-accessibility");
  if (btn) btn.disabled = true;
  setAccessibilityStatus("Waiting for permission…");

  let elapsed = 0;
  const POLL_INTERVAL_MS = 1000;
  const TIMEOUT_MS = 30000;

  accessibilityPollInterval = setInterval(async () => {
    elapsed += POLL_INTERVAL_MS;
    try {
      const granted = await invoke("check_accessibility_permission");
      if (granted) {
        stopAccessibilityPolling();
        if (btn) btn.disabled = false;
        setAccessibilityStatus("Accessibility permission granted ✓");
        showStep("step-microphone");
        return;
      }
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error checking accessibility permission:", e);
    }
    if (elapsed >= TIMEOUT_MS) {
      stopAccessibilityPolling();
      if (btn) btn.disabled = false;
      setAccessibilityStatus(
        "Permission not yet granted. Voice typing will not work until you grant it in System Settings.",
        true
      );
    }
  }, POLL_INTERVAL_MS);
}

const btnGrantAccessibility = document.getElementById("btn-grant-accessibility");
if (btnGrantAccessibility) {
  btnGrantAccessibility.addEventListener("click", async () => {
    try {
      await invoke("request_accessibility_permission");
      await startAccessibilityPolling();
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error requesting accessibility permission:", e);
    }
  });
}

const btnSkipAccessibility = document.getElementById("btn-skip-accessibility");
if (btnSkipAccessibility) {
  btnSkipAccessibility.addEventListener("click", () => {
    stopAccessibilityPolling();
    setAccessibilityStatus(
      "Skipped. Voice typing will not work until Accessibility permission is granted in System Settings.",
      true
    );
    showStep("step-microphone");
  });
}

// ── Microphone Step ───────────────────────────────────────────────────────────

let microphonePollInterval = null;

function stopMicrophonePolling() {
  if (microphonePollInterval !== null) {
    clearInterval(microphonePollInterval);
    microphonePollInterval = null;
  }
}

function setMicrophoneStatus(message, isWarning = false, { withSettingsLink = false } = {}) {
  const statusEl = document.getElementById("microphone-status");
  if (!statusEl) return;
  statusEl.className = isWarning ? "status-warning" : "status-info";
  if (withSettingsLink) {
    statusEl.textContent = message + " ";
    const link = document.createElement("a");
    link.href = "#";
    link.textContent = "Open System Settings";
    link.addEventListener("click", async (e) => {
      e.preventDefault();
      try { await invoke("request_microphone_permission"); } catch (_) { /* logged elsewhere */ }
    });
    statusEl.appendChild(link);
  } else {
    statusEl.textContent = message;
  }
}

async function startMicrophonePolling() {
  stopMicrophonePolling();
  const btn = document.getElementById("btn-grant-microphone");
  if (btn) btn.disabled = true;
  setMicrophoneStatus("Waiting for permission…");

  let elapsed = 0;
  const POLL_INTERVAL_MS = 1000;
  const TIMEOUT_MS = 30000;

  microphonePollInterval = setInterval(async () => {
    elapsed += POLL_INTERVAL_MS;
    try {
      const granted = await invoke("check_microphone_permission");
      if (granted) {
        stopMicrophonePolling();
        if (btn) btn.disabled = false;
        setMicrophoneStatus("Microphone permission granted ✓");
        showStep("step-hotkey");
      } else if (elapsed >= TIMEOUT_MS) {
        stopMicrophonePolling();
        if (btn) btn.disabled = false;
        setMicrophoneStatus(
          "Permission not yet granted. Voice typing will not work until you grant it.",
          true,
          { withSettingsLink: true }
        );
      }
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error checking microphone permission:", e);
    }
  }, POLL_INTERVAL_MS);
}

const btnGrantMicrophone = document.getElementById("btn-grant-microphone");
if (btnGrantMicrophone) {
  btnGrantMicrophone.addEventListener("click", async () => {
    try {
      await invoke("request_microphone_permission");
      await startMicrophonePolling();
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error requesting microphone permission:", e);
    }
  });
}

const btnSkipMicrophone = document.getElementById("btn-skip-microphone");
if (btnSkipMicrophone) {
  btnSkipMicrophone.addEventListener("click", () => {
    stopMicrophonePolling();
    setMicrophoneStatus(
      "Skipped. Voice typing will not work until Microphone permission is granted.",
      true,
      { withSettingsLink: true }
    );
    showStep("step-hotkey");
  });
}

// ── Hotkey Step ───────────────────────────────────────────────────────────────

const DEFAULT_HOTKEY_CODE = "AltRight";

// Modifier-only key codes — pressing these alone is a valid PTT hotkey.
const MODIFIER_CODES = new Set([
  "AltLeft", "AltRight",
  "MetaLeft", "MetaRight",
  "ControlLeft", "ControlRight",
  "ShiftLeft", "ShiftRight",
]);

// Human-readable labels for common key codes.
const KEY_DISPLAY_MAP = {
  AltLeft: "⌥ Left Option",
  AltRight: "⌥ Right Option",
  MetaLeft: "⌘ Left Command",
  MetaRight: "⌘ Right Command",
  ControlLeft: "⌃ Left Control",
  ControlRight: "⌃ Right Control",
  ShiftLeft: "⇧ Left Shift",
  ShiftRight: "⇧ Right Shift",
  Space: "Space",
  Enter: "Return",
  Backspace: "Backspace",
};

// Modifier name → symbol for combo display (e.g. "Alt+Space" → "⌥ Space")
const MODIFIER_SYMBOLS = {
  Alt: "⌥",
  Meta: "⌘",
  Control: "⌃",
  Shift: "⇧",
};

function formatHotkeyCode(code) {
  // Single key (including standalone modifiers)
  if (KEY_DISPLAY_MAP[code]) return KEY_DISPLAY_MAP[code];
  // Modifier+key combo like "Alt+Space"
  const parts = code.split("+");
  return parts
    .map((p) => MODIFIER_SYMBOLS[p] || KEY_DISPLAY_MAP[p] || p)
    .join(" ");
}

let capturedHotkeyCode = null;
let isCapturing = false;

const captureField = document.getElementById("hotkey-capture-field");
const hotkeyDisplay = document.getElementById("hotkey-display");
const btnConfirmHotkey = document.getElementById("btn-confirm-hotkey");
const btnSkipHotkey = document.getElementById("btn-skip-hotkey");

function setCapturedHotkey(code) {
  capturedHotkeyCode = code;
  if (hotkeyDisplay) hotkeyDisplay.textContent = formatHotkeyCode(code);
  if (btnConfirmHotkey) btnConfirmHotkey.disabled = false;
}

if (captureField) {
  captureField.addEventListener("click", () => {
    isCapturing = true;
    captureField.textContent = "Press a key combination…";
    captureField.classList.add("capturing");
  });
}

function handleHotkeyKeydown(e) {
  if (!isCapturing) return;
  // Escape cancels capture; Tab is reserved for navigation
  if (e.key === "Escape" || e.key === "Tab") {
    isCapturing = false;
    if (captureField) {
      captureField.classList.remove("capturing");
      captureField.textContent = capturedHotkeyCode
        ? "Click to change"
        : "Click to capture hotkey";
    }
    return;
  }
  e.preventDefault();
  e.stopPropagation();

  let code;
  if (MODIFIER_CODES.has(e.code)) {
    // Solo modifier key — store the sided code directly (e.g. "AltRight")
    code = e.code;
  } else {
    // Build modifier prefix from active modifier keys (logical, not sided)
    const mods = [];
    if (e.altKey) mods.push("Alt");
    if (e.metaKey) mods.push("Meta");
    if (e.ctrlKey) mods.push("Control");
    if (e.shiftKey) mods.push("Shift");
    code = mods.length > 0 ? mods.join("+") + "+" + e.code : e.code;
  }

  isCapturing = false;
  if (captureField) {
    captureField.classList.remove("capturing");
    captureField.textContent = "Click to change";
  }
  setCapturedHotkey(code);
}

document.addEventListener("keydown", handleHotkeyKeydown);

if (btnConfirmHotkey) {
  btnConfirmHotkey.addEventListener("click", async () => {
    if (!capturedHotkeyCode) return;
    try {
      await invoke("save_ptt_hotkey", { hotkey: capturedHotkeyCode });
      console.debug("[ICantSpell] onboarding: hotkey saved:", capturedHotkeyCode);
      showStep("step-validation");
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error saving hotkey:", e);
    }
  });
}

if (btnSkipHotkey) {
  btnSkipHotkey.addEventListener("click", async () => {
    try {
      await invoke("save_ptt_hotkey", { hotkey: DEFAULT_HOTKEY_CODE });
      console.debug("[ICantSpell] onboarding: default hotkey saved:", DEFAULT_HOTKEY_CODE);
      showStep("step-validation");
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error saving default hotkey:", e);
    }
  });
}

// ── Validation Step ───────────────────────────────────────────────────────────

const validationStatusEl = document.getElementById("validation-status");
const btnFinish = document.getElementById("btn-finish");
const btnFinishAnyway = document.getElementById("btn-finish-anyway");

async function loadValidationStep() {
  if (validationStatusEl) validationStatusEl.innerHTML = "";
  if (btnFinish) btnFinish.style.display = "none";
  if (btnFinishAnyway) btnFinishAnyway.style.display = "none";

  let status;
  try {
    status = await invoke("check_all_permissions");
  } catch (e) {
    console.warn("[ICantSpell] onboarding: error checking permissions:", e);
    if (validationStatusEl) {
      validationStatusEl.innerHTML =
        '<p class="status-warning">Could not check permissions. You can still finish setup.</p>';
    }
    if (btnFinishAnyway) btnFinishAnyway.style.display = "";
    return;
  }

  const items = [
    { label: "Accessibility", granted: status.accessibility },
    { label: "Microphone", granted: status.microphone },
  ];

  if (validationStatusEl) {
    validationStatusEl.innerHTML = items
      .map(
        (item) =>
          `<div class="permission-item">` +
          `<span class="${item.granted ? "granted" : "missing"}">${item.granted ? "✓" : "✗"}</span>` +
          `<span>${item.label}: ${item.granted ? "Granted" : "Not granted"}</span>` +
          `</div>`
      )
      .join("");
  }

  const allGranted = status.accessibility && status.microphone;
  if (allGranted) {
    if (btnFinish) btnFinish.style.display = "";
  } else {
    if (btnFinishAnyway) btnFinishAnyway.style.display = "";
  }
}

// Watch for the validation step becoming active and load it automatically.
const validationStepEl = document.getElementById("step-validation");
if (validationStepEl) {
  new MutationObserver(() => {
    if (validationStepEl.classList.contains("active")) {
      loadValidationStep();
    }
  }).observe(validationStepEl, { attributes: true, attributeFilter: ["class"] });
}

if (btnFinish) {
  btnFinish.addEventListener("click", async () => {
    btnFinish.disabled = true;
    try {
      await invoke("finish_onboarding", { allGranted: true });
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error finishing onboarding:", e);
    }
  });
}

if (btnFinishAnyway) {
  btnFinishAnyway.addEventListener("click", async () => {
    btnFinishAnyway.disabled = true;
    try {
      await invoke("finish_onboarding", { allGranted: false });
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error finishing onboarding:", e);
    }
  });
}
