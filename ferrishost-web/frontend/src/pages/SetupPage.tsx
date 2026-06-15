import { createResource, createSignal } from "solid-js";
import { fetchSetupState, updateSetupState } from "../lib/api";

export function SetupPage() {
  const [state, { refetch }] = createResource(fetchSetupState);
  const [saving, setSaving] = createSignal(false);

  const handleComplete = async () => {
    setSaving(true);
    try {
      await updateSetupState({ ...state()!, completed: true });
      refetch();
    } catch (e) {
      console.error("Failed to save setup state:", e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div class="max-w-2xl mx-auto p-6">
      <h1 class="text-2xl font-bold mb-6">Setup FerrisHost</h1>

      <div class="bg-white rounded-lg border p-6 space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">Domain</label>
          <input
            type="text"
            class="w-full border rounded px-3 py-2"
            placeholder="ferrishost.homelab.local"
            value={state()?.domain ?? ""}
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">TLS Mode</label>
          <select class="w-full border rounded px-3 py-2">
            <option value="self-signed" selected={state()?.tls_mode === "self-signed"}>
              Self-Signed
            </option>
            <option value="lets-encrypt" selected={state()?.tls_mode === "lets-encrypt"}>
              Let's Encrypt
            </option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Admin Username</label>
          <input
            type="text"
            class="w-full border rounded px-3 py-2"
            placeholder="admin"
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Password</label>
          <input type="password" class="w-full border rounded px-3 py-2" placeholder="••••••••" />
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Timezone</label>
          <input
            type="text"
            class="w-full border rounded px-3 py-2"
            placeholder="UTC"
            value={state()?.timezone ?? ""}
          />
        </div>

        <button
          onClick={handleComplete}
          disabled={saving()}
          class="w-full bg-blue-600 text-white rounded px-4 py-2 font-medium hover:bg-blue-700 disabled:opacity-50"
        >
          {saving() ? "Saving..." : "Complete Setup"}
        </button>
      </div>
    </div>
  );
}
