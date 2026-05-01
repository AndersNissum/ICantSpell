// Menu bar popover logic — IPC listeners and controls added in Story 5.1
import { listen } from "@tauri-apps/api/event";

console.debug("[ICantSpell] main window loaded");

// ── Permission Revocation Monitoring ─────────────────────────────────────────

const revokedPermissions = new Set();

function updatePermissionWarning() {
  const el = document.getElementById("permission-warning");
  if (!el) return;
  if (revokedPermissions.size === 0) {
    el.style.display = "none";
    el.textContent = "";
    return;
  }
  const names = [...revokedPermissions].join(" and ");
  el.textContent = `⚠ ${names} permission revoked. Re-grant in System Settings to restore voice mode.`;
  el.style.display = "";
}

listen("permission_revoked", (event) => {
  revokedPermissions.add(event.payload.permissionName);
  updatePermissionWarning();
}).catch((e) => console.warn("[ICantSpell] Failed to listen for permission_revoked:", e));

listen("permission_restored", (event) => {
  revokedPermissions.delete(event.payload.permissionName);
  updatePermissionWarning();
}).catch((e) => console.warn("[ICantSpell] Failed to listen for permission_restored:", e));
