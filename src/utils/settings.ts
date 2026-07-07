import { LazyStore } from "@tauri-apps/plugin-store";

// This automatically creates a 'settings.json' file in the app's config directory
const store = new LazyStore("settings.json");

// Define your hardcoded default values here
const DEFAULTS: Record<string, any> = {
  projectPath: "",
};

export const AppSettings = {
  // Get a value (returns default if not set yet)
  async get(key: string): Promise<any> {
    const value = await store.get(key);
    return value !== null ? value : DEFAULTS[key];
  },

  // Save a value permanently
  async set(key: string, value: any): Promise<void> {
    await store.set(key, value);
    await store.save(); // Crucial: Writes changes instantly to the disk
  },

  // Reset ONE specific variable back to its default state
  async reset(key: string): Promise<void> {
    await store.set(key, DEFAULTS[key]);
    await store.save();
  },

  // FACTORY RESET: Resets ALL variables back to hardcoded defaults
  async resetAll(): Promise<void> {
    for (const key of Object.keys(DEFAULTS)) {
      await store.set(key, DEFAULTS[key]);
    }
    await store.save();
  }
};